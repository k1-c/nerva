use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::types::{RiskTier, ToolMetadata};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyDecision {
    Allow,
    RequireConfirmation,
    Deny(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PolicyConfig {
    pub auto_approve_safe: bool,
    pub auto_approve_caution: bool,
    pub blocked_tools: HashSet<String>,
}

impl Default for PolicyConfig {
    fn default() -> Self {
        Self {
            auto_approve_safe: true,
            auto_approve_caution: false,
            blocked_tools: HashSet::new(),
        }
    }
}

pub struct PolicyEngine {
    config: PolicyConfig,
}

impl PolicyEngine {
    pub fn new(config: PolicyConfig) -> Self {
        Self { config }
    }

    pub fn evaluate(&self, metadata: &ToolMetadata) -> PolicyDecision {
        if self.config.blocked_tools.contains(&metadata.id) {
            return PolicyDecision::Deny(format!("Tool '{}' is blocked by policy", metadata.id));
        }

        match metadata.risk {
            RiskTier::Safe => {
                if self.config.auto_approve_safe {
                    PolicyDecision::Allow
                } else {
                    PolicyDecision::RequireConfirmation
                }
            }
            RiskTier::Caution => {
                if self.config.auto_approve_caution {
                    PolicyDecision::Allow
                } else {
                    PolicyDecision::RequireConfirmation
                }
            }
            RiskTier::Dangerous => PolicyDecision::RequireConfirmation,
        }
    }
}

impl Default for PolicyEngine {
    fn default() -> Self {
        Self::new(PolicyConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn safe_tool() -> ToolMetadata {
        ToolMetadata {
            id: "test_safe".into(),
            name: "Test Safe".into(),
            description: "A safe test tool".into(),
            risk: RiskTier::Safe,
            confirmation_required: false,
        }
    }

    #[test]
    fn test_safe_tool_auto_approved() {
        let engine = PolicyEngine::default();
        assert_eq!(engine.evaluate(&safe_tool()), PolicyDecision::Allow);
    }

    #[test]
    fn test_blocked_tool_denied() {
        let config = PolicyConfig {
            blocked_tools: HashSet::from(["test_safe".into()]),
            ..Default::default()
        };
        let engine = PolicyEngine::new(config);
        assert!(matches!(engine.evaluate(&safe_tool()), PolicyDecision::Deny(_)));
    }
}
