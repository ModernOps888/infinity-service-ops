use crate::primitives::{
    KnowledgeArticleId, ProblemId, RecordMeta, TeamId, TenantId, Timestamp, UserId,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArticleType {
    HowTo,
    Troubleshooting,
    KnownError,
    Policy,
    Runbook,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArticleStatus {
    Draft,
    InReview,
    Published,
    Retired,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArticleAudience {
    Internal,
    TenantUsers,
    Public,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeArticle {
    pub id: KnowledgeArticleId,
    pub tenant_id: TenantId,
    pub title: String,
    pub article_type: ArticleType,
    pub status: ArticleStatus,
    pub audience: ArticleAudience,
    pub owner_team_id: TeamId,
    pub body_storage_ref: String,
    pub source_problem_id: Option<ProblemId>,
    pub review_due_at: Option<Timestamp>,
    pub policy_hook_ids: Vec<String>,
    pub meta: RecordMeta,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeFeedback {
    pub id: String,
    pub tenant_id: TenantId,
    pub article_id: KnowledgeArticleId,
    pub user_id: Option<UserId>,
    pub helpful: bool,
    pub comment: Option<String>,
    pub created_at: Timestamp,
}
