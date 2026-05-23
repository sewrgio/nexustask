use crate::domain::custom_field::{CustomField, CustomFieldType};
use crate::ports::repository::CustomFieldRepository;
use uuid::Uuid;
use anyhow::Result;

pub struct CustomFieldService {
    custom_field_repo: Box<dyn CustomFieldRepository>,
}

impl CustomFieldService {
    pub fn new(custom_field_repo: Box<dyn CustomFieldRepository>) -> Self {
        Self { custom_field_repo }
    }

    pub fn define_field(
        &self,
        workspace_id: Uuid,
        field_name: String,
        field_type: CustomFieldType,
        field_options: Option<String>,
        is_required: bool,
    ) -> Result<CustomField> {
        // Validate field name
        if field_name.is_empty() {
            return Err(anyhow::anyhow!("Field name cannot be empty"));
        }

        // Validate field options for select type
        if field_type == CustomFieldType::Select {
            if field_options.is_none() || field_options.as_ref().unwrap().is_empty() {
                return Err(anyhow::anyhow!("Select fields must have options"));
            }
        }

        let id = self.custom_field_repo.create(
            workspace_id,
            &field_name,
            field_type.as_str(),
            field_options.as_deref(),
            is_required,
        )?;

        Ok(CustomField {
            id,
            workspace_id,
            field_name,
            field_type,
            field_options,
            is_required,
            created_at: chrono::Utc::now(),
        })
    }

    pub fn get_field(&self, field_id: Uuid) -> Result<Option<CustomField>> {
        self.custom_field_repo.find_by_id(field_id)
    }

    pub fn get_workspace_fields(&self, workspace_id: Uuid) -> Result<Vec<CustomField>> {
        self.custom_field_repo.get_workspace_fields(workspace_id)
    }

    pub fn update_field(
        &self,
        field_id: Uuid,
        field_name: String,
        field_options: Option<String>,
        is_required: bool,
    ) -> Result<()> {
        if field_name.is_empty() {
            return Err(anyhow::anyhow!("Field name cannot be empty"));
        }

        self.custom_field_repo.update(field_id, &field_name, field_options.as_deref(), is_required)
    }

    pub fn delete_field(&self, field_id: Uuid) -> Result<()> {
        self.custom_field_repo.delete(field_id)
    }

    pub fn validate_field_value(&self, field_id: Uuid, value: &str) -> Result<()> {
        let field = self.custom_field_repo.find_by_id(field_id)?
            .ok_or_else(|| anyhow::anyhow!("Field not found"))?;

        field.validate_value(value).map_err(|e| anyhow::anyhow!(e))
    }
}
