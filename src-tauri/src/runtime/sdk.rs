//! Minimum node-sdk contract defaults for builtin nodes (v3+).
//! v4 adds lifecycle hook trait (no-op for builtins until v16).

use super::lifecycle::{Lifecycle, LifecycleMode};
use super::protocol::presets::{HowPreset, PortDeclaration, WhatPreset};

pub struct BuiltinNodeDefaults;

impl BuiltinNodeDefaults {
    /// Default I/O declaration inherited by minimal nodes: any payload, single delivery.
    pub fn any_io() -> PortDeclaration {
        PortDeclaration::new(WhatPreset::Any, HowPreset::Single)
    }
}

/// Lifecycle callbacks bound to state transitions (v4 seed).
pub trait LifecycleHooks {
    fn on_transition(
        &mut self,
        _from: Lifecycle,
        _to: Lifecycle,
        _mode: LifecycleMode,
    ) {
    }
    fn on_init(&mut self) {}
    fn on_run(&mut self) {}
    fn on_stop(&mut self) {}
}

pub struct NoopLifecycleHooks;

impl LifecycleHooks for NoopLifecycleHooks {}
