use platform_domain::knowledge::{ArticleAudience, ArticleStatus, KnowledgeArticle};
use platform_domain::primitives::KnowledgeArticleId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchDocument {
    pub id: KnowledgeArticleId,
    pub title: String,
    pub body_excerpt: String,
    pub audience: ArticleAudience,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Citation {
    pub article_id: KnowledgeArticleId,
    pub title: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchResult {
    pub citations: Vec<Citation>,
    pub answer: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SearchIndex {
    pub documents: Vec<SearchDocument>,
}

/// Sensitivity rank for an audience. Higher means more restricted.
/// A viewer may only see documents whose sensitivity is at or below their own clearance.
fn audience_rank(audience: &ArticleAudience) -> u8 {
    match audience {
        ArticleAudience::Public => 0,
        ArticleAudience::TenantUsers => 1,
        ArticleAudience::Internal => 2,
    }
}

impl SearchIndex {
    pub fn add_article(&mut self, article: &KnowledgeArticle, body_excerpt: impl Into<String>) {
        if article.status == ArticleStatus::Published {
            self.documents.push(SearchDocument {
                id: article.id.clone(),
                title: article.title.clone(),
                body_excerpt: body_excerpt.into(),
                audience: article.audience.clone(),
            });
        }
    }

    /// Safe-by-default query: only returns public documents.
    /// Use [`SearchIndex::query_for_audience`] to search with an explicit, authorized clearance.
    pub fn query(&self, search_term: &str) -> SearchResult {
        self.query_for_audience(search_term, &ArticleAudience::Public)
    }

    /// Audience-aware query. Documents more sensitive than `viewer_clearance` are never
    /// returned, preventing internal runbooks/known-errors from leaking to lower-trust callers.
    pub fn query_for_audience(
        &self,
        search_term: &str,
        viewer_clearance: &ArticleAudience,
    ) -> SearchResult {
        let normalized = search_term.to_ascii_lowercase();
        let clearance = audience_rank(viewer_clearance);
        let matches: Vec<_> = self
            .documents
            .iter()
            .filter(|document| audience_rank(&document.audience) <= clearance)
            .filter(|document| {
                document.title.to_ascii_lowercase().contains(&normalized)
                    || document
                        .body_excerpt
                        .to_ascii_lowercase()
                        .contains(&normalized)
            })
            .collect();

        let citations = matches
            .iter()
            .map(|document| Citation {
                article_id: document.id.clone(),
                title: document.title.clone(),
            })
            .collect::<Vec<_>>();

        let answer = if matches.is_empty() {
            "No matching knowledge found.".to_string()
        } else {
            matches
                .iter()
                .map(|document| document.body_excerpt.clone())
                .collect::<Vec<_>>()
                .join(" ")
        };

        SearchResult { citations, answer }
    }
}

#[cfg(test)]
mod tests {
    use super::SearchIndex;
    use platform_domain::knowledge::{
        ArticleAudience, ArticleStatus, ArticleType, KnowledgeArticle,
    };
    use platform_domain::primitives::KnowledgeArticleId;
    use platform_domain::{ActorRef, RecordMeta, TeamId, TenantId};

    #[test]
    fn query_returns_only_published_articles_with_citations() {
        let tenant_id = TenantId::new("tenant-a");
        let actor = ActorRef::system("knowledge");
        let article = KnowledgeArticle {
            id: KnowledgeArticleId::new("kb-1"),
            tenant_id: tenant_id.clone(),
            title: "Reset MFA device".to_string(),
            article_type: ArticleType::HowTo,
            status: ArticleStatus::Published,
            audience: ArticleAudience::TenantUsers,
            owner_team_id: TeamId::new("identity"),
            body_storage_ref: "object://kb-1".to_string(),
            source_problem_id: None,
            review_due_at: None,
            policy_hook_ids: vec![],
            meta: RecordMeta::bootstrap(tenant_id, actor),
        };

        let mut index = SearchIndex::default();
        index.add_article(
            &article,
            "Use the self-service portal to register a new MFA device.",
        );

        let result = index.query_for_audience("mfa device", &ArticleAudience::TenantUsers);
        assert_eq!(result.citations.len(), 1);
        assert!(result.answer.contains("self-service portal"));
    }

    fn published_article(id: &str, title: &str, audience: ArticleAudience) -> KnowledgeArticle {
        let tenant_id = TenantId::new("tenant-a");
        let actor = ActorRef::system("knowledge");
        KnowledgeArticle {
            id: KnowledgeArticleId::new(id),
            tenant_id: tenant_id.clone(),
            title: title.to_string(),
            article_type: ArticleType::Runbook,
            status: ArticleStatus::Published,
            audience,
            owner_team_id: TeamId::new("secops"),
            body_storage_ref: format!("object://{id}"),
            source_problem_id: None,
            review_due_at: None,
            policy_hook_ids: vec![],
            meta: RecordMeta::bootstrap(tenant_id, actor),
        }
    }

    #[test]
    fn internal_articles_are_hidden_from_lower_trust_viewers() {
        let mut index = SearchIndex::default();
        index.add_article(
            &published_article(
                "kb-int",
                "Domain admin recovery runbook",
                ArticleAudience::Internal,
            ),
            "Break-glass steps for domain admin recovery.",
        );
        index.add_article(
            &published_article(
                "kb-pub",
                "Domain password self-service",
                ArticleAudience::Public,
            ),
            "Public domain password reset guidance.",
        );

        // A tenant user must not see the internal runbook.
        let tenant_view = index.query_for_audience("domain", &ArticleAudience::TenantUsers);
        assert_eq!(tenant_view.citations.len(), 1);
        assert_eq!(
            tenant_view.citations[0].article_id,
            KnowledgeArticleId::new("kb-pub")
        );
        assert!(!tenant_view.answer.contains("Break-glass"));

        // An internal operator sees both.
        let internal_view = index.query_for_audience("domain", &ArticleAudience::Internal);
        assert_eq!(internal_view.citations.len(), 2);
    }

    #[test]
    fn default_query_is_public_only() {
        let mut index = SearchIndex::default();
        index.add_article(
            &published_article("kb-int", "Internal only", ArticleAudience::Internal),
            "Sensitive internal content.",
        );

        let result = index.query("internal");
        assert!(result.citations.is_empty());
        assert_eq!(result.answer, "No matching knowledge found.");
    }
}
