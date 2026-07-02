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

    pub fn query(&self, search_term: &str) -> SearchResult {
        let normalized = search_term.to_ascii_lowercase();
        let matches: Vec<_> = self
            .documents
            .iter()
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

        let result = index.query("mfa device");
        assert_eq!(result.citations.len(), 1);
        assert!(result.answer.contains("self-service portal"));
    }
}
