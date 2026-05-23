use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CustomFieldType {
    Text,
    Number,
    Date,
    Boolean,
    Select,
}

impl CustomFieldType {
    pub fn as_str(&self) -> &'static str {
        match self {
            CustomFieldType::Text => "text",
            CustomFieldType::Number => "number",
            CustomFieldType::Date => "date",
            CustomFieldType::Boolean => "boolean",
            CustomFieldType::Select => "select",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "text" => CustomFieldType::Text,
            "number" => CustomFieldType::Number,
            "date" => CustomFieldType::Date,
            "boolean" => CustomFieldType::Boolean,
            "select" => CustomFieldType::Select,
            _ => CustomFieldType::Text,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomField {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub field_name: String,
    pub field_type: CustomFieldType,
    pub field_options: Option<String>, // JSON string for select options
    pub is_required: bool,
    pub created_at: DateTime<Utc>,
}

impl CustomField {
    pub fn new(
        workspace_id: Uuid,
        field_name: String,
        field_type: CustomFieldType,
        field_options: Option<String>,
        is_required: bool,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            workspace_id,
            field_name,
            field_type,
            field_options,
            is_required,
            created_at: Utc::now(),
        }
    }

    pub fn validate_value(&self, value: &str) -> Result<(), String> {
        if self.is_required && value.is_empty() {
            return Err(format!("Field '{}' is required", self.field_name));
        }

        match self.field_type {
            CustomFieldType::Number => {
                if !value.is_empty() {
                    value.parse::<f64>()
                        .map_err(|_| format!("Field '{}' must be a number", self.field_name))?;
                }
            }
            CustomFieldType::Boolean => {
                if !value.is_empty() && value != "true" && value != "false" {
                    return Err(format!("Field '{}' must be true or false", self.field_name));
                }
            }
            CustomFieldType::Select => {
                if let Some(options) = &self.field_options {
                    if !value.is_empty() {
                        let options_vec: Vec<&str> = options.split(',').map(|s| s.trim()).collect();
                        if !options_vec.contains(&value) {
                            return Err(format!("Field '{}' must be one of: {}", self.field_name, options));
                        }
                    }
                }
            }
            CustomFieldType::Date => {
                if !value.is_empty() {
                    chrono::DateTime::parse_from_rfc3339(value)
                        .map_err(|_| format!("Field '{}' must be a valid ISO 8601 date", self.field_name))?;
                }
            }
            CustomFieldType::Text => {
                // Text fields accept any string
            }
        }

        Ok(())
    }
}
