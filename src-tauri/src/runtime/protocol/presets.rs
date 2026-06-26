use std::collections::HashMap;
use std::fmt;

use serde::{Deserialize, Serialize};

pub const PROTOCOL_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Axis {
    What,
    How,
}

impl fmt::Display for Axis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Axis::What => write!(f, "What"),
            Axis::How => write!(f, "How"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "preset", rename_all = "kebab-case")]
pub enum WhatPreset {
    Any,
    Text,
    Json,
    Bytes,
    Custom { id: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "preset", rename_all = "kebab-case")]
pub enum HowPreset {
    Single,
    Stream,
    #[serde(rename = "request-response")]
    RequestResponse,
    Broadcast,
    Custom { id: String },
}

impl WhatPreset {
    pub fn id(&self) -> &str {
        match self {
            WhatPreset::Any => "any",
            WhatPreset::Text => "text",
            WhatPreset::Json => "json",
            WhatPreset::Bytes => "bytes",
            WhatPreset::Custom { .. } => "custom",
        }
    }

    pub fn is_custom(&self) -> bool {
        matches!(self, WhatPreset::Custom { .. })
    }
}

impl HowPreset {
    pub fn id(&self) -> &str {
        match self {
            HowPreset::Single => "single",
            HowPreset::Stream => "stream",
            HowPreset::RequestResponse => "request-response",
            HowPreset::Broadcast => "broadcast",
            HowPreset::Custom { .. } => "custom",
        }
    }

    pub fn is_custom(&self) -> bool {
        matches!(self, HowPreset::Custom { .. })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortDeclaration {
    pub what: WhatPreset,
    pub how: HowPreset,
}

impl PortDeclaration {
    pub fn new(what: WhatPreset, how: HowPreset) -> Self {
        Self { what, how }
    }

    pub fn from_ids(what: &str, how: &str) -> Result<Self, String> {
        let what = match what {
            "any" => WhatPreset::Any,
            "text" => WhatPreset::Text,
            "json" => WhatPreset::Json,
            "bytes" => WhatPreset::Bytes,
            other => return Err(format!("unknown What preset: {other}")),
        };
        let how = match how {
            "single" => HowPreset::Single,
            "stream" => HowPreset::Stream,
            "request-response" => HowPreset::RequestResponse,
            "broadcast" => HowPreset::Broadcast,
            other => return Err(format!("unknown How preset: {other}")),
        };
        Ok(Self::new(what, how))
    }

    pub fn label(&self) -> String {
        format!("{}·{}", self.what.id(), self.how.id())
    }
}

pub fn default_port_decls_for_kind(kind: &crate::runtime::node::NodeKind) -> HashMap<String, PortDeclaration> {
    use crate::runtime::edge::{PORT_IN, PORT_OUT};
    use crate::runtime::node::NodeKind;

    let text_single = PortDeclaration::new(WhatPreset::Text, HowPreset::Single);
    let json_single = PortDeclaration::new(WhatPreset::Json, HowPreset::Single);

    let mut decls = HashMap::new();
    match kind {
        NodeKind::Constant { .. } => {
            decls.insert(PORT_OUT.to_string(), text_single);
        }
        NodeKind::JsonConstant { .. } => {
            decls.insert(PORT_OUT.to_string(), json_single);
        }
        NodeKind::Echo { .. } => {
            decls.insert(PORT_IN.to_string(), text_single.clone());
            decls.insert(PORT_OUT.to_string(), text_single);
        }
    }
    decls
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::node::NodeKind;

    #[test]
    fn constant_defaults_text_single_out() {
        let decls = default_port_decls_for_kind(&NodeKind::Constant {
            value: "x".into(),
        });
        let out = decls.get("out").unwrap();
        assert_eq!(out.what, WhatPreset::Text);
        assert_eq!(out.how, HowPreset::Single);
    }

    #[test]
    fn json_constant_defaults_json_single_out() {
        let decls = default_port_decls_for_kind(&NodeKind::JsonConstant {
            value: "{}".into(),
        });
        let out = decls.get("out").unwrap();
        assert_eq!(out.what, WhatPreset::Json);
    }
}
