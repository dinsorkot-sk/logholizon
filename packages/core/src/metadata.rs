use serde::{Deserialize, Serialize};

pub const CANONICAL_FIELD_TYPES: &[&str] = &[
    "text",
    "long_text",
    "number",
    "decimal",
    "currency",
    "percentage",
    "integer",
    "boolean",
    "date",
    "datetime",
    "time",
    "select",
    "multi_select",
    "email",
    "phone",
    "url",
    "json",
    "file",
    "image",
    "reference",
    "computed",
    "formula",
    "auto_number",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FieldType {
    Text,
    LongText,
    Number,
    Decimal,
    Currency,
    Percentage,
    Integer,
    Boolean,
    Date,
    DateTime,
    Time,
    Select,
    MultiSelect,
    Email,
    Phone,
    Url,
    Json,
    File,
    Image,
    Reference,
    Computed,
    Formula,
    AutoNumber,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FieldMetadata {
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub required: bool,
    #[serde(default, rename = "unique")]
    pub unique: bool,
    #[serde(default)]
    pub default: Option<serde_json::Value>,
    #[serde(default)]
    pub readonly: bool,
    #[serde(default)]
    pub hidden: bool,
    #[serde(default)]
    pub searchable: bool,
    #[serde(default)]
    pub sortable: bool,
    #[serde(default)]
    pub filterable: bool,
    #[serde(default)]
    pub indexed: bool,
    #[serde(default)]
    pub precision: Option<u8>,
    #[serde(default)]
    pub min: Option<f64>,
    #[serde(default)]
    pub max: Option<f64>,
    #[serde(default)]
    pub regex: Option<String>,
    #[serde(default)]
    pub help_text: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EntityMetadata {
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub settings: serde_json::Value,
}

pub fn validate_field_type(field_type: &str) -> anyhow::Result<()> {
    if CANONICAL_FIELD_TYPES.contains(&field_type) || matches!(field_type, "textarea" | "checkbox")
    {
        return Ok(());
    }
    Err(crate::error::AppError::BadRequest(format!("invalid field type: {field_type}")).into())
}

pub fn is_text_type(field_type: &str) -> bool {
    matches!(
        field_type,
        "text" | "long_text" | "textarea" | "email" | "phone" | "url"
    )
}

pub fn is_numeric_type(field_type: &str) -> bool {
    matches!(
        field_type,
        "number" | "decimal" | "currency" | "percentage" | "integer"
    )
}

pub fn is_boolean_type(field_type: &str) -> bool {
    matches!(field_type, "boolean" | "checkbox")
}

pub fn is_option_type(field_type: &str) -> bool {
    matches!(field_type, "select" | "multi_select")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_types_cover_phase_two() {
        assert_eq!(CANONICAL_FIELD_TYPES.len(), 23);
        assert!(CANONICAL_FIELD_TYPES.contains(&"datetime"));
        assert!(CANONICAL_FIELD_TYPES.contains(&"multi_select"));
        assert!(CANONICAL_FIELD_TYPES.contains(&"formula"));
    }

    #[test]
    fn metadata_round_trips() {
        let metadata = FieldMetadata {
            label: Some("Plate Number".into()),
            required: true,
            searchable: true,
            precision: Some(2),
            help_text: Some("Vehicle registration plate".into()),
            ..Default::default()
        };
        let value = serde_json::to_value(&metadata).unwrap();
        let decoded: FieldMetadata = serde_json::from_value(value).unwrap();
        assert!(decoded.required && decoded.searchable);
        assert_eq!(decoded.precision, Some(2));
    }
}
