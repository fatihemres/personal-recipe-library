use super::AppError;
use serde::{Deserialize, Serialize};
use unicode_normalization::UnicodeNormalization;

pub fn invalid() -> AppError {
    AppError::new("INVALID_INPUT", "errors.validation", true)
}
pub fn conflict() -> AppError {
    AppError::new("CONFLICT", "errors.conflict", true)
}
pub fn not_found() -> AppError {
    AppError::new("NOT_FOUND", "errors.notFound", true)
}
pub fn valid_id(id: &str) -> Result<(), AppError> {
    let parsed = uuid::Uuid::parse_str(id).map_err(|_| invalid())?;
    if parsed.to_string() != id {
        return Err(invalid());
    }
    Ok(())
}
pub fn search_name(name: &str) -> String {
    name.nfc()
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .flat_map(|c| match c {
            'I' => 'ı'.to_lowercase(),
            'İ' => 'i'.to_lowercase(),
            _ => c.to_lowercase(),
        })
        .collect()
}
pub fn decimal(value: &str) -> bool {
    let mut parts = value.split('.');
    let whole = parts.next().unwrap_or("");
    let fraction = parts.next();
    !whole.is_empty()
        && whole.len() <= 12
        && whole.bytes().all(|c| c.is_ascii_digit())
        && (whole == "0" || !whole.starts_with('0'))
        && parts.next().is_none()
        && fraction
            .is_none_or(|f| !f.is_empty() && f.len() <= 6 && f.bytes().all(|c| c.is_ascii_digit()))
        && value.bytes().any(|c| (b'1'..=b'9').contains(&c))
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Ingredient {
    pub id: String,
    pub name: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Unit {
    pub code: String,
    pub dimension: String,
    pub canonical_code: String,
    pub factor: i64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RecipeIngredient {
    pub id: String,
    pub ingredient_id: String,
    pub quantity: Option<String>,
    pub unit_code: String,
    pub note: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RecipeStep {
    pub id: String,
    pub instructions: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RecipeInput {
    pub id: String,
    pub expected_revision: Option<i64>,
    pub title: String,
    pub description: Option<String>,
    pub kind: String,
    pub servings: String,
    pub prep_minutes: Option<i64>,
    pub cook_minutes: Option<i64>,
    pub notes: Option<String>,
    pub ingredients: Vec<RecipeIngredient>,
    pub steps: Vec<RecipeStep>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Recipe {
    pub ingredient_names: std::collections::BTreeMap<String, String>,
    pub id: String,
    pub revision: i64,
    pub title: String,
    pub description: Option<String>,
    pub kind: String,
    pub servings: String,
    pub prep_minutes: Option<i64>,
    pub cook_minutes: Option<i64>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub ingredients: Vec<RecipeIngredient>,
    pub steps: Vec<RecipeStep>,
}
impl RecipeInput {
    pub fn validate(&self) -> Result<(), AppError> {
        valid_id(&self.id)?;
        let text = |s: &str, max: usize| {
            !s.trim().is_empty() && s.chars().count() <= max && !s.contains('\0')
        };
        if !text(&self.title, 200)
            || !["food", "beverage"].contains(&self.kind.as_str())
            || !decimal(&self.servings)
            || self.expected_revision.is_some_and(|r| r < 1)
            || self
                .description
                .as_ref()
                .is_some_and(|s| s.chars().count() > 10000 || s.contains('\0'))
            || self
                .notes
                .as_ref()
                .is_some_and(|s| s.chars().count() > 20000 || s.contains('\0'))
            || [self.prep_minutes, self.cook_minutes]
                .iter()
                .flatten()
                .any(|v| !(0..=10080).contains(v))
            || self.ingredients.len() > 500
            || self.steps.len() > 500
        {
            return Err(invalid());
        }
        let mut ids = std::collections::HashSet::new();
        for i in &self.ingredients {
            valid_id(&i.id)?;
            valid_id(&i.ingredient_id)?;
            if !ids.insert(&i.id)
                || i.quantity.as_ref().is_some_and(|q| !decimal(q))
                || i.note
                    .as_ref()
                    .is_some_and(|s| s.chars().count() > 2000 || s.contains('\0'))
            {
                return Err(invalid());
            }
        }
        for s in &self.steps {
            valid_id(&s.id)?;
            if !ids.insert(&s.id) || !text(&s.instructions, 10000) {
                return Err(invalid());
            }
        }
        Ok(())
    }
}
