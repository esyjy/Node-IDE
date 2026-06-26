use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Lifecycle {
    Created,
    Running,
    Done,
    Failed,
}

#[derive(Debug, Error, PartialEq, Eq)]
#[error("invalid lifecycle transition: {from:?} -> {to:?}")]
pub struct LifecycleError {
    pub from: Lifecycle,
    pub to: Lifecycle,
}

impl Lifecycle {
    pub fn can_transition(from: Lifecycle, to: Lifecycle) -> bool {
        matches!(
            (from, to),
            (Lifecycle::Created, Lifecycle::Running)
                | (Lifecycle::Running, Lifecycle::Done)
                | (Lifecycle::Running, Lifecycle::Failed)
                | (Lifecycle::Done, Lifecycle::Running)
                | (Lifecycle::Failed, Lifecycle::Running)
        )
    }

    pub fn transition(&mut self, to: Lifecycle) -> Result<(), LifecycleError> {
        if !Self::can_transition(*self, to) {
            return Err(LifecycleError {
                from: *self,
                to,
            });
        }
        *self = to;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn happy_path_created_running_done() {
        let mut lc = Lifecycle::Created;
        assert!(lc.transition(Lifecycle::Running).is_ok());
        assert!(lc.transition(Lifecycle::Done).is_ok());
    }

    #[test]
    fn rerun_from_done() {
        let mut lc = Lifecycle::Done;
        assert!(lc.transition(Lifecycle::Running).is_ok());
        assert!(lc.transition(Lifecycle::Done).is_ok());
    }

    #[test]
    fn reject_created_to_done() {
        let mut lc = Lifecycle::Created;
        assert!(lc.transition(Lifecycle::Done).is_err());
    }
}
