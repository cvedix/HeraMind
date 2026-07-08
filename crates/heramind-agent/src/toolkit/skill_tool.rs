//! Skill management tool for querying and managing operation guides.

use async_trait::async_trait;
use serde_json::Value;

use super::error::{Result, ToolError};
use super::object_schema;
use super::tool::{Tool, ToolCategory};
use super::ToolOutput;
use crate::skills;

/// Tool for managing operation guides (skills).
///
/// Skills are reusable step-by-step guides built from available tools.
pub struct SkillTool {
    registry: skills::SharedSkillRegistry,
    data_dir: Option<std::path::PathBuf>,
}

impl SkillTool {
    /// Create a new skill tool.
    pub fn new(registry: skills::SharedSkillRegistry) -> Self {
        Self {
            registry,
            data_dir: None,
        }
    }

    /// Create a skill tool with persistence support.
    pub fn with_data_dir(
        registry: skills::SharedSkillRegistry,
        data_dir: std::path::PathBuf,
    ) -> Self {
        Self {
            registry,
            data_dir: Some(data_dir),
        }
    }

    /// Validate a skill ID contains only safe characters.
    fn is_safe_id(id: &str) -> bool {
        !id.is_empty()
            && id.len() <= 128
            && id
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    }

    /// Persist a skill file to disk.
    fn persist(&self, id: &str, content: &str) {
        if let Some(ref dir) = self.data_dir {
            let skills_dir = dir.join("skills");
            let _ = std::fs::create_dir_all(&skills_dir);
            let path = skills_dir.join(format!("{}.md", id));
            if let Err(e) = std::fs::write(&path, content) {
                tracing::error!(path = %path.display(), error = %e, "Failed to persist skill");
            }
        }
    }

    /// Delete a skill file from disk.
    fn remove_file(&self, id: &str) {
        if let Some(ref dir) = self.data_dir {
            let path = dir.join("skills").join(format!("{}.md", id));
            if path.exists() {
                if let Err(e) = std::fs::remove_file(&path) {
                    tracing::error!(path = %path.display(), error = %e, "Failed to delete skill file");
                }
            }
        }
    }
}

#[async_trait]
impl Tool for SkillTool {
    fn name(&self) -> &str {
        "skill"
    }

    fn description(&self) -> &str {
        r##"Load operation guides (skills) when you need them. Skills contain step-by-step instructions, CLI command examples, and common error solutions for specific scenarios.

IMPORTANT: Skills are NOT in your system prompt. You MUST call this tool to load a skill guide BEFORE performing operations you're unfamiliar with.

Actions:
- search: Search skills by query keywords — returns matching skill IDs and descriptions. Use this first to find the right skill.
- load: Load a skill's full guide content by ID — returns the complete step-by-step guide. Call this after search, or when you know the skill ID.
- create: Create a new user skill (requires 'content' with YAML frontmatter + Markdown body)
- update: Update an existing skill by ID (full content replacement)
- delete: Delete a user skill by ID

Available skill IDs (load these when relevant):
- device-onboarding: Device connection, MQTT, webhook, drafts
- dashboard-management: Dashboard CRUD, widget layout, data binding
- rule-management: Rule DSL, triggers, actions, CRUD
- agent-management: AI Agent CRUD, scheduling, execution modes
- message-management: Message sending, channel configuration
- transform-management: Data transform CRUD, JS code
- extension-development: Extension development, FFI, build
- widget-development: Custom widget creation, manifest, bundle
- connector-management: External MQTT broker connections
- data-push-management: Data push to external systems
- llm-management: LLM backend CRUD, capability, default selection

When to load a skill:
- User asks to create/update/delete any entity → load the relevant skill FIRST
- You're unsure about CLI command syntax → load the skill for that domain
- A command fails and you need troubleshooting steps → load the skill for error solutions"##
    }

    fn parameters(&self) -> Value {
        object_schema(
            serde_json::json!({
                "action": {
                    "type": "string",
                    "enum": ["search", "load", "create", "update", "delete"],
                    "description": "Operation to perform. Use 'search' to find relevant skills, 'load' to read a skill guide."
                },
                "id": {
                    "type": "string",
                    "description": "Skill ID for load/update/delete. Example: 'rule-management', 'device-onboarding'"
                },
                "query": {
                    "type": "string",
                    "description": "Search query for finding relevant skills. Example: 'device', 'rule', 'dashboard'"
                },
                "content": {
                    "type": "string",
                    "description": "Full skill file content for create/update (YAML frontmatter + Markdown body)."
                }
            }),
            vec!["action".to_string()],
        )
    }

    fn category(&self) -> ToolCategory {
        ToolCategory::System
    }

    async fn execute(&self, args: Value) -> Result<ToolOutput> {
        let action = args["action"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidArguments("action is required".into()))?;

        match action {
            "search" => {
                let query = args["query"]
                    .as_str()
                    .or_else(|| args["id"].as_str())
                    .unwrap_or("");

                let registry_guard = self.registry.read().await;
                let query_lower = query.to_lowercase();

                // Score all skills against the query
                let mut results: Vec<(String, String, f32)> = Vec::new();
                for skill in registry_guard.list() {
                    let mut score = 0.0f32;
                    // Keyword match
                    for keyword in &skill.metadata.triggers.keywords {
                        let kw_lower = keyword.to_lowercase();
                        if query_lower.contains(&kw_lower) || kw_lower.contains(&query_lower) {
                            score += 1.0;
                        }
                    }
                    // ID/name match
                    if skill.metadata.id.to_lowercase().contains(&query_lower)
                        || query_lower.contains(&skill.metadata.id.to_lowercase())
                    {
                        score += 2.0;
                    }
                    // Category match
                    if format!("{:?}", skill.metadata.category).to_lowercase().contains(&query_lower) {
                        score += 0.5;
                    }

                    if score > 0.0 {
                        // Extract first non-empty line from body as description
                        let desc = skill.body
                            .lines()
                            .find(|l| !l.is_empty() && !l.starts_with('#'))
                            .unwrap_or("Step-by-step guide")
                            .chars()
                            .take(100)
                            .collect::<String>();
                        results.push((skill.metadata.id.clone(), desc, score));
                    }
                }

                if results.is_empty() {
                    // Return all skills as fallback
                    let all: Vec<String> = registry_guard.list()
                        .iter()
                        .map(|s| format!("- {}: {}", s.metadata.id, s.metadata.name))
                        .collect();
                    Ok(ToolOutput::success(serde_json::json!({
                        "message": "No specific match. All available skills:",
                        "skills": all,
                        "hint": "Use action='load' with one of these IDs to get the full guide."
                    })))
                } else {
                    results.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
                    let skills: Vec<serde_json::Value> = results.iter()
                        .take(5)
                        .map(|(id, desc, score)| serde_json::json!({
                            "id": id,
                            "description": desc,
                            "relevance": format!("{:.1}", score)
                        }))
                        .collect();
                    Ok(ToolOutput::success(serde_json::json!({
                        "matches": skills,
                        "hint": "Use action='load' and 'id' to get the full guide."
                    })))
                }
            }
            "load" => {
                let id = args["id"]
                    .as_str()
                    .ok_or_else(|| ToolError::InvalidArguments("id is required for load".into()))?;

                let registry_guard = self.registry.read().await;
                match registry_guard.get(id) {
                    Some(skill) => {
                        Ok(ToolOutput::success(serde_json::json!({
                            "id": skill.metadata.id,
                            "name": skill.metadata.name,
                            "guide": skill.body,
                        })))
                    }
                    None => {
                        // Suggest similar skills
                        let suggestions: Vec<String> = registry_guard.list()
                            .iter()
                            .filter(|s| {
                                let sid = s.metadata.id.to_lowercase();
                                let qid = id.to_lowercase();
                                sid.contains(&qid) || qid.contains(&sid)
                            })
                            .map(|s| s.metadata.id.clone())
                            .take(3)
                            .collect();
                        let mut msg = format!("Skill '{}' not found.", id);
                        if !suggestions.is_empty() {
                            msg.push_str(&format!(" Did you mean: {}?", suggestions.join(", ")));
                        }
                        Ok(ToolOutput::error(msg))
                    }
                }
            }
            "create" => {
                let content = args["content"].as_str().ok_or_else(|| {
                    ToolError::InvalidArguments(
                        "content is required for create (YAML frontmatter + Markdown body)".into(),
                    )
                })?;

                let mut registry_guard = self.registry.write().await;
                match registry_guard.add_user_skill(content) {
                    Ok(id) => {
                        let skill = registry_guard.get(&id)
                            .ok_or_else(|| ToolError::Execution("Skill created but not found in registry".into()))?;
                        self.persist(&id, content);
                        Ok(ToolOutput::success(serde_json::json!({
                            "id": skill.metadata.id,
                            "name": skill.metadata.name,
                            "category": format!("{:?}", skill.metadata.category).to_lowercase(),
                            "message": format!("Skill '{}' created successfully", id),
                        })))
                    }
                    Err(e) => Ok(ToolOutput::error(format!("Failed to create skill. Check YAML frontmatter format and try again. Error: {}", e))),
                }
            }
            "update" => {
                let id = args["id"].as_str().ok_or_else(|| {
                    ToolError::InvalidArguments("id is required for update".into())
                })?;
                let content = args["content"].as_str().ok_or_else(|| {
                    ToolError::InvalidArguments(
                        "content is required for update (YAML frontmatter + Markdown body)".into(),
                    )
                })?;

                let mut registry_guard = self.registry.write().await;
                match registry_guard.update_user_skill(id, content) {
                    Ok(()) => {
                        let skill = registry_guard.get(id)
                            .ok_or_else(|| ToolError::Execution("Skill updated but not found in registry".into()))?;
                        self.persist(id, content);
                        Ok(ToolOutput::success(serde_json::json!({
                            "id": skill.metadata.id,
                            "name": skill.metadata.name,
                            "message": format!("Skill '{}' updated successfully", id),
                        })))
                    }
                    Err(e) => Ok(ToolOutput::error(format!("Failed to update skill. Check YAML frontmatter format and try again. Error: {}", e))),
                }
            }
            "delete" => {
                let id = args["id"].as_str().ok_or_else(|| {
                    ToolError::InvalidArguments("id is required for delete".into())
                })?;

                if !Self::is_safe_id(id) {
                    return Ok(ToolOutput::error(format!("Invalid skill ID '{}'", id)));
                }

                let mut registry_guard = self.registry.write().await;
                match registry_guard.delete_skill(id) {
                    Ok(skill) => {
                        self.remove_file(id);
                        Ok(ToolOutput::success(serde_json::json!({
                            "message": format!("Skill '{}' ('{}') deleted successfully", id, skill.metadata.name),
                        })))
                    }
                    Err(e) => Ok(ToolOutput::error(format!("Failed to delete skill. Error: {}", e))),
                }
            }
            _ => Err(ToolError::InvalidArguments(format!(
                "Unknown action '{}' for skill. Available actions: search, load, create, update, delete",
                action
            ))),
        }
    }
}
