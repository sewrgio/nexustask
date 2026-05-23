use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct FileAttachment {
    pub id: Uuid,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub user_id: Uuid,
    pub workspace_id: Uuid,
    pub file_name: String,
    pub file_path: String,
    pub file_size: i64,
    pub mime_type: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl FileAttachment {
    pub fn new(
        entity_type: String,
        entity_id: Uuid,
        user_id: Uuid,
        workspace_id: Uuid,
        file_name: String,
        file_path: String,
        file_size: i64,
        mime_type: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            entity_type,
            entity_id,
            user_id,
            workspace_id,
            file_name,
            file_path,
            file_size,
            mime_type,
            created_at: Utc::now(),
        }
    }

    pub fn validate_entity_type(entity_type: &str) -> Result<(), String> {
        if !matches!(entity_type, "task" | "comment") {
            return Err("Entity type must be 'task' or 'comment'".to_string());
        }
        Ok(())
    }

    pub fn validate_file_size(file_size: i64) -> Result<(), String> {
        const MAX_FILE_SIZE: i64 = 100 * 1024 * 1024; // 100 MB
        if file_size <= 0 {
            return Err("File size must be positive".to_string());
        }
        if file_size > MAX_FILE_SIZE {
            return Err("File size exceeds 100 MB limit".to_string());
        }
        Ok(())
    }

    pub fn validate_file_name(file_name: &str) -> Result<(), String> {
        if file_name.is_empty() {
            return Err("File name cannot be empty".to_string());
        }
        if file_name.len() > 255 {
            return Err("File name must be less than 255 characters".to_string());
        }
        Ok(())
    }

    pub fn is_valid(&self) -> bool {
        Self::validate_entity_type(&self.entity_type).is_ok()
            && Self::validate_file_size(self.file_size).is_ok()
            && Self::validate_file_name(&self.file_name).is_ok()
    }

    pub fn get_extension(&self) -> Option<String> {
        self.file_name
            .rsplit('.')
            .next()
            .map(|ext| ext.to_lowercase())
    }

    pub fn is_image(&self) -> bool {
        matches!(self.get_extension().as_deref(), Some("jpg" | "jpeg" | "png" | "gif" | "webp" | "svg"))
    }

    pub fn is_document(&self) -> bool {
        matches!(self.get_extension().as_deref(), Some("pdf" | "doc" | "docx" | "txt" | "md"))
    }
}
