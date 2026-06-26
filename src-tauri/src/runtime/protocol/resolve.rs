use super::presets::{Axis, HowPreset, PortDeclaration, WhatPreset};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolutionOutcome {
    Compatible,
    Reject {
        axis: Axis,
        reason: String,
        hint: String,
    },
}

pub fn resolve_ports(
    source: &PortDeclaration,
    target: &PortDeclaration,
) -> ResolutionOutcome {
    match resolve_what(&source.what, &target.what) {
        ResolutionOutcome::Compatible => {}
        reject @ ResolutionOutcome::Reject { .. } => return reject,
    }

    resolve_how(&source.how, &target.how)
}

fn resolve_what(source: &WhatPreset, target: &WhatPreset) -> ResolutionOutcome {
    if source.is_custom() || target.is_custom() {
        return reject_what(
            source,
            target,
            "Custom What presets are not supported in v3.",
            "Use a built-in What preset (any, text, json, bytes).",
        );
    }

    if matches!(source, WhatPreset::Any) || matches!(target, WhatPreset::Any) {
        return ResolutionOutcome::Compatible;
    }

    if source == target {
        return ResolutionOutcome::Compatible;
    }

    reject_what(
        source,
        target,
        &format!(
            "What mismatch: source {} cannot connect to target {}.",
            source.id(),
            target.id()
        ),
        &format!(
            "Change the target port to {} or the source port to {}.",
            source.id(),
            target.id()
        ),
    )
}

fn resolve_how(source: &HowPreset, target: &HowPreset) -> ResolutionOutcome {
    if source.is_custom() || target.is_custom() {
        return reject_how(
            source,
            target,
            "Custom How presets are not supported in v3.",
            "Use a built-in How preset (single, stream, request-response, broadcast).",
        );
    }

    if matches!(source, HowPreset::Single) && matches!(target, HowPreset::Single) {
        return ResolutionOutcome::Compatible;
    }

    reject_how(
        source,
        target,
        &format!(
            "How mismatch: {} → {} is not supported yet.",
            source.id(),
            target.id()
        ),
        "Only single → single connections are supported in v3 (channels in v10, adapters in v13).",
    )
}

fn reject_what(source: &WhatPreset, target: &WhatPreset, reason: &str, hint: &str) -> ResolutionOutcome {
    let _ = (source, target);
    ResolutionOutcome::Reject {
        axis: Axis::What,
        reason: reason.to_string(),
        hint: hint.to_string(),
    }
}

fn reject_how(source: &HowPreset, target: &HowPreset, reason: &str, hint: &str) -> ResolutionOutcome {
    let _ = (source, target);
    ResolutionOutcome::Reject {
        axis: Axis::How,
        reason: reason.to_string(),
        hint: hint.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::protocol::presets::{HowPreset, PortDeclaration, WhatPreset};

    fn decl(what: WhatPreset, how: HowPreset) -> PortDeclaration {
        PortDeclaration::new(what, how)
    }

    fn single() -> HowPreset {
        HowPreset::Single
    }

    #[test]
    fn text_single_to_text_single_compatible() {
        let source = decl(WhatPreset::Text, single());
        let target = decl(WhatPreset::Text, single());
        assert_eq!(resolve_ports(&source, &target), ResolutionOutcome::Compatible);
    }

    #[test]
    fn json_to_text_rejected_on_what_axis() {
        let source = decl(WhatPreset::Json, single());
        let target = decl(WhatPreset::Text, single());
        match resolve_ports(&source, &target) {
            ResolutionOutcome::Reject { axis, reason, .. } => {
                assert_eq!(axis, Axis::What);
                assert!(reason.contains("json"));
                assert!(reason.contains("text"));
            }
            _ => panic!("expected reject"),
        }
    }

    #[test]
    fn any_matches_text() {
        let source = decl(WhatPreset::Any, single());
        let target = decl(WhatPreset::Text, single());
        assert_eq!(resolve_ports(&source, &target), ResolutionOutcome::Compatible);
    }

    #[test]
    fn stream_to_single_rejected_on_how_axis() {
        let source = decl(WhatPreset::Text, HowPreset::Stream);
        let target = decl(WhatPreset::Text, single());
        match resolve_ports(&source, &target) {
            ResolutionOutcome::Reject { axis, reason, .. } => {
                assert_eq!(axis, Axis::How);
                assert!(reason.contains("stream"));
            }
            _ => panic!("expected reject"),
        }
    }

    #[test]
    fn what_grid_exhaustive_non_custom() {
        let presets = [
            WhatPreset::Any,
            WhatPreset::Text,
            WhatPreset::Json,
            WhatPreset::Bytes,
        ];

        for source in &presets {
            for target in &presets {
                let outcome = resolve_what(source, target);
                let expected_compatible =
                    matches!(source, WhatPreset::Any)
                        || matches!(target, WhatPreset::Any)
                        || source == target;
                assert_eq!(
                    outcome == ResolutionOutcome::Compatible,
                    expected_compatible,
                    "what {source:?} -> {target:?}"
                );
            }
        }
    }

    #[test]
    fn how_grid_exhaustive_non_custom() {
        let presets = [
            HowPreset::Single,
            HowPreset::Stream,
            HowPreset::RequestResponse,
            HowPreset::Broadcast,
        ];

        for source in &presets {
            for target in &presets {
                let outcome = resolve_how(source, target);
                let expected_compatible =
                    matches!(source, HowPreset::Single) && matches!(target, HowPreset::Single);
                assert_eq!(
                    outcome == ResolutionOutcome::Compatible,
                    expected_compatible,
                    "how {source:?} -> {target:?}"
                );
            }
        }
    }
}
