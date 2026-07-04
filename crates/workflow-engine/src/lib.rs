use platform_core::{ActorRef, Timestamp};

/// Default retry budget applied to every workflow step unless a definition overrides it.
pub const DEFAULT_MAX_STEP_ATTEMPTS: u32 = 3;

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
    pub approved_by: Option<ActorRef>,
}

impl ApprovalGate {
    pub fn is_approved(&self) -> bool {
        self.approved_at.is_some()
    }
}

/// Runtime guardrail violations. Every enforcement failure is a typed, auditable value
/// rather than a silently ignored call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GuardrailViolation {
    UnknownApprovalGate {
        approval_id: String,
    },
    ApproverRoleMismatch {
        approval_id: String,
        required_role: String,
    },
    /// Separation of duties: the actor that started an execution may not approve it.
    SelfApprovalNotAllowed {
        approval_id: String,
    },
    ApprovalAlreadyGranted {
        approval_id: String,
    },
    ExecutionNotRunning {
        status: ExecutionStatus,
    },
    StepOutOfRange {
        index: usize,
    },
    StepAlreadyCompleted {
        index: usize,
    },
    RetryBudgetExhausted {
        index: usize,
        max_attempts: u32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowDefinition {
    pub id: String,
    pub name: String,
    pub trigger: WorkflowTrigger,
    pub actions: Vec<WorkflowAction>,
    pub approvals: Vec<ApprovalGate>,
    /// Per-step retry budget enforced at runtime.
    pub max_step_attempts: u32,
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
    /// Approval gates cloned from the definition; every gate must be individually
    /// satisfied before the execution leaves `WaitingApproval`.
    pub approvals: Vec<ApprovalGate>,
    pub max_step_attempts: u32,
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
            approvals: self.approvals.clone(),
            max_step_attempts: self.max_step_attempts.max(1),
            history: vec!["workflow-started".to_string()],
        }
    }
}

impl WorkflowExecution {
    /// Grant one approval gate. Enforced at runtime:
    /// - the gate must exist on this execution
    /// - the approver must hold the gate's required role
    /// - the initiating actor may not approve their own execution (separation of duties)
    /// - a gate may only be approved once (no replay)
    ///
    /// The execution only transitions to `Running` once **all** gates are approved.
    pub fn approve(
        &mut self,
        approval_id: &str,
        approver: &ActorRef,
        approver_roles: &[String],
        approved_at: Timestamp,
    ) -> Result<(), GuardrailViolation> {
        if approver == &self.actor {
            return Err(GuardrailViolation::SelfApprovalNotAllowed {
                approval_id: approval_id.to_string(),
            });
        }

        let gate = self
            .approvals
            .iter_mut()
            .find(|gate| gate.id == approval_id)
            .ok_or_else(|| GuardrailViolation::UnknownApprovalGate {
                approval_id: approval_id.to_string(),
            })?;

        if gate.is_approved() {
            return Err(GuardrailViolation::ApprovalAlreadyGranted {
                approval_id: approval_id.to_string(),
            });
        }

        if !approver_roles
            .iter()
            .any(|role| role == &gate.approver_role)
        {
            return Err(GuardrailViolation::ApproverRoleMismatch {
                approval_id: approval_id.to_string(),
                required_role: gate.approver_role.clone(),
            });
        }

        gate.approved_at = Some(approved_at.clone());
        gate.approved_by = Some(approver.clone());
        self.history.push(format!(
            "approval:{}:by:{}@{}",
            approval_id, approver.display_name, approved_at.0
        ));

        if self.approvals.iter().all(ApprovalGate::is_approved) {
            self.status = ExecutionStatus::Running;
            self.history.push("all-approvals-granted".to_string());
        }

        Ok(())
    }

    /// Complete a step. Rejected unless the execution is `Running`, so approval
    /// gates can no longer be bypassed by driving steps directly.
    pub fn mark_step_complete(&mut self, index: usize) -> Result<(), GuardrailViolation> {
        if self.status != ExecutionStatus::Running {
            return Err(GuardrailViolation::ExecutionNotRunning {
                status: self.status.clone(),
            });
        }

        let step = self
            .steps
            .get_mut(index)
            .ok_or(GuardrailViolation::StepOutOfRange { index })?;

        if step.completed {
            return Err(GuardrailViolation::StepAlreadyCompleted { index });
        }

        step.completed = true;
        step.attempts += 1;
        self.history.push(format!("step-{}-completed", index));

        if self.steps.iter().all(|step| step.completed) {
            self.status = ExecutionStatus::Succeeded;
            self.history.push("workflow-succeeded".to_string());
        }

        Ok(())
    }

    /// Retry a step within the per-step retry budget. Rejected while the execution
    /// is waiting for approval, and the budget cannot be exceeded; exhausting it
    /// marks the execution `Failed`.
    pub fn retry_step(&mut self, index: usize) -> Result<(), GuardrailViolation> {
        if !matches!(
            self.status,
            ExecutionStatus::Running | ExecutionStatus::Failed
        ) {
            return Err(GuardrailViolation::ExecutionNotRunning {
                status: self.status.clone(),
            });
        }

        let max_attempts = self.max_step_attempts;
        let step = self
            .steps
            .get_mut(index)
            .ok_or(GuardrailViolation::StepOutOfRange { index })?;

        if step.completed {
            return Err(GuardrailViolation::StepAlreadyCompleted { index });
        }

        if step.attempts >= max_attempts {
            self.status = ExecutionStatus::Failed;
            self.history
                .push(format!("retry-budget-exhausted-step-{}", index));
            return Err(GuardrailViolation::RetryBudgetExhausted {
                index,
                max_attempts,
            });
        }

        step.attempts += 1;
        self.status = ExecutionStatus::Running;
        self.history.push(format!("retry-step-{}", index));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ApprovalGate, DEFAULT_MAX_STEP_ATTEMPTS, ExecutionStatus, GuardrailViolation,
        WorkflowAction, WorkflowDefinition, WorkflowTrigger,
    };
    use platform_core::{ActorRef, Timestamp};

    fn gate(id: &str, role: &str) -> ApprovalGate {
        ApprovalGate {
            id: id.to_string(),
            approver_role: role.to_string(),
            approved_at: None,
            approved_by: None,
        }
    }

    fn gated_definition(approvals: Vec<ApprovalGate>) -> WorkflowDefinition {
        WorkflowDefinition {
            id: "wf-1".to_string(),
            name: "Emergency change approval".to_string(),
            trigger: WorkflowTrigger::StateChanged,
            actions: vec![WorkflowAction::CreateTask, WorkflowAction::SendNotification],
            approvals,
            max_step_attempts: DEFAULT_MAX_STEP_ATTEMPTS,
        }
    }

    fn now() -> Timestamp {
        Timestamp::new("1970-01-01T00:00:00Z")
    }

    #[test]
    fn approval_gate_blocks_execution_until_approved() {
        let definition = gated_definition(vec![gate("approval-1", "cab")]);
        let mut execution = definition.start(ActorRef::system("worker"));
        assert_eq!(execution.status, ExecutionStatus::WaitingApproval);

        let approver = ActorRef::system("cab-chair");
        execution
            .approve("approval-1", &approver, &["cab".to_string()], now())
            .expect("valid approval should be accepted");
        execution.mark_step_complete(0).expect("step 0 completes");
        execution.mark_step_complete(1).expect("step 1 completes");

        assert_eq!(execution.status, ExecutionStatus::Succeeded);
    }

    #[test]
    fn steps_cannot_complete_while_waiting_for_approval() {
        let definition = gated_definition(vec![gate("approval-1", "cab")]);
        let mut execution = definition.start(ActorRef::system("worker"));

        let result = execution.mark_step_complete(0);
        assert_eq!(
            result,
            Err(GuardrailViolation::ExecutionNotRunning {
                status: ExecutionStatus::WaitingApproval,
            })
        );
        assert!(!execution.steps[0].completed);
    }

    #[test]
    fn unknown_approval_gate_is_rejected() {
        let definition = gated_definition(vec![gate("approval-1", "cab")]);
        let mut execution = definition.start(ActorRef::system("worker"));

        let approver = ActorRef::system("cab-chair");
        let result = execution.approve("forged-gate", &approver, &["cab".to_string()], now());
        assert_eq!(
            result,
            Err(GuardrailViolation::UnknownApprovalGate {
                approval_id: "forged-gate".to_string(),
            })
        );
        assert_eq!(execution.status, ExecutionStatus::WaitingApproval);
    }

    #[test]
    fn approver_without_required_role_is_rejected() {
        let definition = gated_definition(vec![gate("approval-1", "cab")]);
        let mut execution = definition.start(ActorRef::system("worker"));

        let approver = ActorRef::system("intern");
        let result = execution.approve("approval-1", &approver, &["viewer".to_string()], now());
        assert_eq!(
            result,
            Err(GuardrailViolation::ApproverRoleMismatch {
                approval_id: "approval-1".to_string(),
                required_role: "cab".to_string(),
            })
        );
        assert_eq!(execution.status, ExecutionStatus::WaitingApproval);
    }

    #[test]
    fn initiator_cannot_approve_own_execution() {
        let definition = gated_definition(vec![gate("approval-1", "cab")]);
        let initiator = ActorRef::system("worker");
        let mut execution = definition.start(initiator.clone());

        let result = execution.approve("approval-1", &initiator, &["cab".to_string()], now());
        assert_eq!(
            result,
            Err(GuardrailViolation::SelfApprovalNotAllowed {
                approval_id: "approval-1".to_string(),
            })
        );
        assert_eq!(execution.status, ExecutionStatus::WaitingApproval);
    }

    #[test]
    fn all_gates_must_be_approved_before_running() {
        let definition = gated_definition(vec![
            gate("approval-1", "cab"),
            gate("approval-2", "security-lead"),
        ]);
        let mut execution = definition.start(ActorRef::system("worker"));

        let cab = ActorRef::system("cab-chair");
        execution
            .approve("approval-1", &cab, &["cab".to_string()], now())
            .expect("first approval accepted");
        assert_eq!(execution.status, ExecutionStatus::WaitingApproval);

        let security = ActorRef::system("security-lead-1");
        execution
            .approve(
                "approval-2",
                &security,
                &["security-lead".to_string()],
                now(),
            )
            .expect("second approval accepted");
        assert_eq!(execution.status, ExecutionStatus::Running);
    }

    #[test]
    fn approval_replay_is_rejected() {
        let definition = gated_definition(vec![gate("approval-1", "cab")]);
        let mut execution = definition.start(ActorRef::system("worker"));

        let approver = ActorRef::system("cab-chair");
        execution
            .approve("approval-1", &approver, &["cab".to_string()], now())
            .expect("first approval accepted");

        let replay = execution.approve("approval-1", &approver, &["cab".to_string()], now());
        assert_eq!(
            replay,
            Err(GuardrailViolation::ApprovalAlreadyGranted {
                approval_id: "approval-1".to_string(),
            })
        );
    }

    #[test]
    fn retrying_a_step_increments_attempts() {
        let definition = WorkflowDefinition {
            id: "wf-2".to_string(),
            name: "Webhook retry".to_string(),
            trigger: WorkflowTrigger::EventIngested,
            actions: vec![WorkflowAction::CallWebhook],
            approvals: vec![],
            max_step_attempts: DEFAULT_MAX_STEP_ATTEMPTS,
        };

        let mut execution = definition.start(ActorRef::system("worker"));
        execution.retry_step(0).expect("retry within budget");

        assert_eq!(execution.steps[0].attempts, 1);
        assert_eq!(execution.status, ExecutionStatus::Running);
    }

    #[test]
    fn retry_cannot_bypass_approval_gate() {
        let definition = gated_definition(vec![gate("approval-1", "cab")]);
        let mut execution = definition.start(ActorRef::system("worker"));

        let result = execution.retry_step(0);
        assert_eq!(
            result,
            Err(GuardrailViolation::ExecutionNotRunning {
                status: ExecutionStatus::WaitingApproval,
            })
        );
        assert_eq!(execution.status, ExecutionStatus::WaitingApproval);
    }

    #[test]
    fn retry_budget_is_enforced() {
        let definition = WorkflowDefinition {
            id: "wf-3".to_string(),
            name: "Webhook retry budget".to_string(),
            trigger: WorkflowTrigger::EventIngested,
            actions: vec![WorkflowAction::CallWebhook],
            approvals: vec![],
            max_step_attempts: 2,
        };

        let mut execution = definition.start(ActorRef::system("worker"));
        execution.retry_step(0).expect("attempt 1");
        execution.retry_step(0).expect("attempt 2");

        let exhausted = execution.retry_step(0);
        assert_eq!(
            exhausted,
            Err(GuardrailViolation::RetryBudgetExhausted {
                index: 0,
                max_attempts: 2,
            })
        );
        assert_eq!(execution.status, ExecutionStatus::Failed);
    }
}
