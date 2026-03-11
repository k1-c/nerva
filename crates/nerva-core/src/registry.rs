use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;

use crate::skill::Skill;
use crate::types::ToolMetadata;

pub struct ToolRegistry {
    skills: RwLock<HashMap<String, Arc<dyn Skill>>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            skills: RwLock::new(HashMap::new()),
        }
    }

    pub async fn register(&self, skill: Arc<dyn Skill>) {
        let id = skill.metadata().id.clone();
        tracing::info!(tool_id = %id, "Registered skill");
        self.skills.write().await.insert(id, skill);
    }

    pub async fn get(&self, id: &str) -> Option<Arc<dyn Skill>> {
        self.skills.read().await.get(id).cloned()
    }

    pub async fn list(&self) -> Vec<ToolMetadata> {
        self.skills
            .read()
            .await
            .values()
            .map(|s| s.metadata().clone())
            .collect()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}
