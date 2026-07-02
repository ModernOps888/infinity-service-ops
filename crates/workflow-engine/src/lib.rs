use platform_core::{ActorRef, Timestamp};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkflowTrigger {
    RecordCreated,
    RecordUpdated,
    StateChanged,
    EventIngested,
    Schedule,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkflowAction {
    Assign,
    ChangeState,
    CreateTask,
    SendNotification,
    CallWebhook,
    CreateIncident,
    AppendAuditTrail,
    EscalateVendor,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApprovalGate {
    pub id: String,
    pub approver_role: String,
    pub approved_at: Option<Timestamp>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowDefinition {
    pub id: String,
    pub name: String,
    pub trigger: WorkflowTrigger,
    pub actions: Vec<WorkflowAction>,
    pub approvals: Vec<ApprovalGate>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionStatus {
    Pending,
    WaitingApproval,
    Running,
    Failed,
    Succeeded,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionStep {
    pub action: WorkflowAction,
    pub attempts: u32,
    pub completed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowExecution {
    pub definition_id: String,
    pub actor: ActorRef,
    pub status: ExecutionStatus,
    pub steps: Vec<ExecutionStep>,
    pub history: Vec<String>,
}

impl WorkflowDefinition {
    pub fn start(&self, actor: ActorRef) -> WorkflowExecution {
        let status = if self.approvals.is_empty() {
            ExecutionStatus::Running
        } else {
            ExecutionStatus::WaitingApproval
        };

        WorkflowExecution {
            definition_id: self.id.clone(),
            actor,
            status,
            steps: self
                .actions
                .iter()
                .cloned()
                .map(|action| ExecutionStep {
                    action,
                    attempts: 0,
                    completed: false,
                })
                .collect(),
            history: vec!["workflow-started".to_string()],
        }
    }
}

impl WorkflowExecution {
    pub fn approve(&mut self, approval_id: &str, approved_at: Timestamp) {
        self.history
            .push(format!("approval:{}@{}", approval_id, approved_at.0));
        self.status = ExecutionStatus::Running;
    }

    pub fn mark_step_complete(&mut self, index: usize) {
        if let Some(step) = self.steps.get_mut(index) {
            step.completed = true;
            step.attempts += 1;
        }

        if self.steps.iter().all(|step| step.completed) {
            self.status = ExecutionStatus::Succeeded;
        }
    }

    pub fn retry_step(&mut self, index: usize) {
        if let Some(step) = self.steps.get_mut(index) {
            step.attempts += 1;
            self.status = ExecutionStatus::Running;
            self.history.push(format!("retry-step-{}", index));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ApprovalGate, ExecutionStatus, WorkflowAction, WorkflowDefinition, WorkflowTrigger,
    };
    use platform_core::{ActorRef, Timestamp};

    #[test]
    fn approval_gate_blocks_execution_until_approved() {
        let definition = WorkflowDefinition {
            id: "wf-1".to_string(),
            name: "Emergency change approval".to_string(),
            trigger: WorkflowTrigger::StateChanged,
            actions: vec![WorkflowAction::CreateTask, WorkflowAction::SendNotification],
            approvals: vec![ApprovalGate {
                id: "approval-1".to_string(),
                approver_role: "cab".to_string(),
                approved_at: None,
            }],
        };

        let mut execution = definition.start(ActorRef::system("worker"));
        assert_eq!(execution.status, ExecutionStatus::WaitingApproval);

        execution.approve("approval-1", Timestamp::new("1970-01-01T00:00:00Z"));
        execution.mark_step_complete(0);
        execution.mark_step_complete(1);

        assert_eq!(execution.status, ExecutionStatus::Succeeded);
    }

    #[test]
    fn retrying_a_step_increments_attempts() {
        let definition = WorkflowDefinition {
            id: "wf-2".to_string(),
            name: "Webhook retry".to_string(),
            trigger: WorkflowTrigger::EventIngested,
            actions: vec![WorkflowAction::CallWebhook],
            approvals: vec![],
        };

        let mut execution = definition.start(ActorRef::system("worker"));
        execution.retry_step(0);

        assert_eq!(execution.steps[0].attempts, 1);
        assert_eq!(execution.status, ExecutionStatus::Running);
    }
}
