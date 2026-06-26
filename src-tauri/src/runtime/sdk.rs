//! Minimum node-sdk contract defaults for builtin nodes (v3).

use super::protocol::presets::{HowPreset, PortDeclaration, WhatPreset};

pub struct BuiltinNodeDefaults;

impl BuiltinNodeDefaults {
    /// Default I/O declaration inherited by minimal nodes: any payload, single delivery.
    pub fn any_io() -> PortDeclaration {
        PortDeclaration::new(WhatPreset::Any, HowPreset::Single)
    }
}
