use super::{recipes::*, AppError};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Draft {
    pub id: String,
    pub revision: i64,
    pub recipe_id: Option<String>,
    pub updated_at: String,
    pub input: RecipeInput,
    pub ingredient_names: std::collections::BTreeMap<String, String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DraftWrite {
    pub id: String,
    pub expected_revision: Option<i64>,
    pub input: RecipeInput,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PersonalIngredient {
    pub id: String,
    pub name: String,
    pub notes: Option<String>,
    pub preferred_unit: Option<String>,
    pub revision: i64,
    pub origin: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IngredientEdit {
    pub id: String,
    pub revision: i64,
    pub name: String,
    pub notes: Option<String>,
    pub preferred_unit: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IngredientSearch {
    pub items: Vec<PersonalIngredient>,
    pub total: i64,
    pub has_more: bool,
}
pub fn draft_conflict() -> AppError {
    AppError::new("DRAFT_CONFLICT", "errors.draftConflict", true)
}
impl DraftWrite {
    pub fn validate(&self) -> Result<(), AppError> {
        valid_id(&self.id)?;
        valid_id(&self.input.id)?;
        let i = &self.input;
        if self.expected_revision.is_some_and(|v| v < 1)
            || i.expected_revision.is_some_and(|v| v < 1)
            || !["food", "beverage"].contains(&i.kind.as_str())
            || i.title.chars().count() > 200
            || i.title.contains('\0')
            || i.servings.len() > 100
            || i.description
                .as_ref()
                .is_some_and(|s| s.chars().count() > 10000)
            || i.notes.as_ref().is_some_and(|s| s.chars().count() > 20000)
            || i.ingredients.len() > 500
            || i.steps.len() > 500
        {
            return Err(invalid());
        }
        let mut ids = std::collections::HashSet::new();
        for line in &i.ingredients {
            valid_id(&line.id)?;
            valid_id(&line.ingredient_id)?;
            if !ids.insert(&line.id)
                || line.quantity.as_ref().is_some_and(|s| s.len() > 100)
                || line.note.as_ref().is_some_and(|s| s.chars().count() > 2000)
            {
                return Err(invalid());
            }
        }
        for step in &i.steps {
            valid_id(&step.id)?;
            if !ids.insert(&step.id) || step.instructions.chars().count() > 10000 {
                return Err(invalid());
            }
        }
        Ok(())
    }
}
impl Recipe {
    pub fn to_input(&self) -> RecipeInput {
        RecipeInput {
            id: self.id.clone(),
            expected_revision: Some(self.revision),
            title: self.title.clone(),
            description: self.description.clone(),
            kind: self.kind.clone(),
            servings: self.servings.clone(),
            prep_minutes: self.prep_minutes,
            cook_minutes: self.cook_minutes,
            notes: self.notes.clone(),
            ingredients: self.ingredients.clone(),
            steps: self.steps.clone(),
        }
    }
}
pub fn independent(mut input: RecipeInput) -> RecipeInput {
    input.id = uuid::Uuid::new_v4().to_string();
    input.expected_revision = None;
    for line in &mut input.ingredients {
        line.id = uuid::Uuid::new_v4().to_string();
    }
    for step in &mut input.steps {
        step.id = uuid::Uuid::new_v4().to_string();
    }
    input
}
