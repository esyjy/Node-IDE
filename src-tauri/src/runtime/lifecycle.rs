use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LifecycleMode {
    Ephemeral,
    Persistent,
}

impl Default for LifecycleMode {
    fn default() -> Self {
        Self::Ephemeral
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Lifecycle {
    #[serde(alias = "created")]
    Idle,
    Initializing,
    Running,
    Waiting,
    Stopping,
    Stopped,
    Done,
    Failed,
}

#[derive(Debug, Error, PartialEq, Eq)]
#[error("invalid lifecycle transition: {from:?} -> {to:?} (mode {mode:?})")]
pub struct LifecycleError {
    pub from: Lifecycle,
    pub to: Lifecycle,
    pub mode: LifecycleMode,
}

impl Lifecycle {
    pub fn initial_for_mode(mode: LifecycleMode) -> Self {
        match mode {
            LifecycleMode::Ephemeral => Lifecycle::Idle,
            LifecycleMode::Persistent => Lifecycle::Stopped,
        }
    }

    pub fn can_transition_with_mode(from: Lifecycle, to: Lifecycle, mode: LifecycleMode) -> bool {
        match mode {
            LifecycleMode::Ephemeral => matches!(
                (from, to),
                (Lifecycle::Idle, Lifecycle::Running)
                    | (Lifecycle::Done, Lifecycle::Running)
                    | (Lifecycle::Failed, Lifecycle::Running)
                    | (Lifecycle::Running, Lifecycle::Done)
                    | (Lifecycle::Running, Lifecycle::Failed)
            ),
            LifecycleMode::Persistent => matches!(
                (from, to),
                (Lifecycle::Stopped, Lifecycle::Initializing)
                    | (Lifecycle::Idle, Lifecycle::Initializing)
                    | (Lifecycle::Initializing, Lifecycle::Idle)
                    | (Lifecycle::Initializing, Lifecycle::Waiting)
                    | (Lifecycle::Idle, Lifecycle::Running)
                    | (Lifecycle::Waiting, Lifecycle::Running)
                    | (Lifecycle::Running, Lifecycle::Idle)
                    | (Lifecycle::Running, Lifecycle::Failed)
                    | (Lifecycle::Idle, Lifecycle::Stopping)
                    | (Lifecycle::Waiting, Lifecycle::Stopping)
                    | (Lifecycle::Running, Lifecycle::Stopping)
                    | (Lifecycle::Stopping, Lifecycle::Stopped)
                    | (Lifecycle::Failed, Lifecycle::Initializing)
            ),
        }
    }

    pub fn transition_with_mode(
        &mut self,
        to: Lifecycle,
        mode: LifecycleMode,
    ) -> Result<(), LifecycleError> {
        if !Self::can_transition_with_mode(*self, to, mode) {
            return Err(LifecycleError {
                from: *self,
                to,
                mode,
            });
        }
        *self = to;
        Ok(())
    }

    /// Post-run terminal state for the given mode.
    pub fn after_success(mode: LifecycleMode) -> Lifecycle {
        match mode {
            LifecycleMode::Ephemeral => Lifecycle::Done,
            LifecycleMode::Persistent => Lifecycle::Idle,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn apply_sequence(
        mode: LifecycleMode,
        sequence: &[Lifecycle],
    ) -> Result<(), LifecycleError> {
        let mut lc = Lifecycle::initial_for_mode(mode);
        for &to in sequence {
            lc.transition_with_mode(to, mode)?;
        }
        Ok(())
    }

    #[test]
    fn ephemeral_happy_path_idle_running_done() {
        let mut lc = Lifecycle::Idle;
        assert!(lc.transition_with_mode(Lifecycle::Running, LifecycleMode::Ephemeral).is_ok());
        assert!(lc.transition_with_mode(Lifecycle::Done, LifecycleMode::Ephemeral).is_ok());
    }

    #[test]
    fn ephemeral_rerun_from_done() {
        let mut lc = Lifecycle::Done;
        assert!(lc.transition_with_mode(Lifecycle::Running, LifecycleMode::Ephemeral).is_ok());
        assert!(lc.transition_with_mode(Lifecycle::Done, LifecycleMode::Ephemeral).is_ok());
    }

    #[test]
    fn ephemeral_reject_idle_to_done() {
        let mut lc = Lifecycle::Idle;
        assert!(lc
            .transition_with_mode(Lifecycle::Done, LifecycleMode::Ephemeral)
            .is_err());
    }

    #[test]
    fn persistent_start_stop_path() {
        apply_sequence(
            LifecycleMode::Persistent,
            &[
                Lifecycle::Initializing,
                Lifecycle::Idle,
                Lifecycle::Running,
                Lifecycle::Idle,
                Lifecycle::Stopping,
                Lifecycle::Stopped,
            ],
        )
        .unwrap();
    }

    #[test]
    fn persistent_start_to_waiting() {
        apply_sequence(
            LifecycleMode::Persistent,
            &[Lifecycle::Initializing, Lifecycle::Waiting],
        )
        .unwrap();
    }

    #[test]
    fn persistent_reject_done_transition() {
        let mut lc = Lifecycle::Idle;
        assert!(lc
            .transition_with_mode(Lifecycle::Done, LifecycleMode::Persistent)
            .is_err());
    }

    #[test]
    fn fuzz_random_sequences_never_panic() {
        use std::collections::HashSet;

        let all = [
            Lifecycle::Idle,
            Lifecycle::Initializing,
            Lifecycle::Running,
            Lifecycle::Waiting,
            Lifecycle::Stopping,
            Lifecycle::Stopped,
            Lifecycle::Done,
            Lifecycle::Failed,
        ];

        for mode in [LifecycleMode::Ephemeral, LifecycleMode::Persistent] {
            for seed in 0..500u64 {
                let mut lc = Lifecycle::initial_for_mode(mode);
                let mut seen = HashSet::from([lc]);
                for step in 0..12usize {
                    let pick = all[((seed as usize).wrapping_mul(31).wrapping_add(step)) % all.len()];
                    if let Ok(()) = lc.transition_with_mode(pick, mode) {
                        seen.insert(lc);
                    }
                }
                assert!(!seen.is_empty());
            }
        }
    }
}
