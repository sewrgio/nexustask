use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct Comment {
    pub id: Uuid,
    pub task_id: Uuid,
    pub user_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub content: String,
    pub data: serde_json::Value,
    pub lft: i64,
    pub rgt: i64,
    pub depth: i32,
    pub workspace_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Comment {
    pub fn new(task_id: Uuid, user_id: Uuid, content: String, workspace_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            task_id,
            user_id,
            parent_id: None,
            content,
            data: serde_json::json!({}),
            lft: 0,
            rgt: 1,
            depth: 0,
            workspace_id,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn validate_content(content: &str) -> Result<(), String> {
        if content.is_empty() {
            return Err("Comment content cannot be empty".to_string());
        }
        if content.len() > 10000 {
            return Err("Comment content must be less than 10000 characters".to_string());
        }
        Ok(())
    }

    pub fn is_root(&self) -> bool {
        self.parent_id.is_none()
    }

    pub fn is_leaf(&self) -> bool {
        self.rgt == self.lft + 1
    }

    pub fn has_replies(&self) -> bool {
        self.rgt > self.lft + 1
    }

    pub fn is_reply_to(&self, parent_id: Uuid) -> bool {
        self.parent_id == Some(parent_id)
    }

    pub fn is_descendant_of(&self, potential_parent: &Comment) -> bool {
        self.lft > potential_parent.lft && self.rgt < potential_parent.rgt
    }

    pub fn is_ancestor_of(&self, potential_child: &Comment) -> bool {
        self.lft < potential_child.lft && self.rgt > potential_child.rgt
    }

    pub fn is_sibling_of(&self, other: &Comment) -> bool {
        self.parent_id == other.parent_id && self.id != other.id
    }

    pub fn can_reply_to(&self, parent_comment: &Comment) -> Result<(), String> {
        if parent_comment.task_id != self.task_id {
            return Err("Cannot reply to comment from different task".to_string());
        }
        if parent_comment.workspace_id != self.workspace_id {
            return Err("Cannot reply to comment from different workspace".to_string());
        }
        Ok(())
    }

    pub fn set_parent(&mut self, parent_id: Uuid) {
        self.parent_id = Some(parent_id);
        self.updated_at = Utc::now();
    }

    pub fn remove_parent(&mut self) {
        self.parent_id = None;
        self.updated_at = Utc::now();
    }

    pub fn update_content(&mut self, content: String) -> Result<(), String> {
        Self::validate_content(&content)?;
        self.content = content;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn extract_mentions(&self) -> Vec<String> {
        let mut mentions = Vec::new();
        for part in self.content.split('@') {
            if let Some(end) = part.find(|c: char| !c.is_alphanumeric() && c != '_') {
                if end > 0 {
                    mentions.push(format!("@{}", &part[..end]));
                }
            } else if !part.is_empty() {
                mentions.push(format!("@{}", part));
            }
        }
        mentions
    }
}
