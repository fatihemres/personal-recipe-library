//! Version-one content envelope. Future formats may introduce stable media asset
//! IDs, never absolute paths; old payloads remain immutable and format-dispatched.
use super::{recipes::*, AppError};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const MAX_SNAPSHOT_BYTES: usize = 128 * 1024 * 1024;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RecipeSnapshot {
    pub format_version: i64,
    pub recipe_id: String,
    pub recipe_created_at: String,
    pub title: String,
    pub description: Option<String>,
    pub kind: String,
    pub servings: String,
    pub prep_minutes: Option<i64>,
    pub cook_minutes: Option<i64>,
    pub notes: Option<String>,
    pub ingredients: Vec<RecipeIngredient>,
    pub ingredient_names: BTreeMap<String, String>,
    pub steps: Vec<RecipeStep>,
}
impl RecipeSnapshot {
    /// Frozen V1 comparison projection, independent of live RecipeInput's future
    /// serialization. New content fields require a new snapshot format/projection.
    pub(crate) fn comparison_v1(&self) -> serde_json::Value {
        use serde_json::json;
        let ingredients:Vec<_>=self.ingredients.iter().map(|i|json!({
            "id":i.id,"ingredientId":i.ingredient_id,"quantity":i.quantity,"unitCode":i.unit_code,"note":i.note
        })).collect();
        let steps:Vec<_>=self.steps.iter().map(|s|{
            let mut lines=s.ingredient_line_ids.clone();lines.sort();
            let details=s.details.as_ref().map(|d|json!({
                "formatVersion":d.format_version,"title":d.title,"durationMinutes":d.duration_minutes,
                "temperatureValue":d.temperature_value,"temperatureUnit":d.temperature_unit,
                "heatLevel":d.heat_level,"equipment":d.equipment,"technique":d.technique,"tips":d.tips
            })).filter(|d|d.as_object().is_some_and(|d|d.iter().any(|(key,v)|key!="formatVersion" && !v.is_null())));
            json!({"id":s.id,"instructions":s.instructions,"details":details,"ingredientLineIds":lines})
        }).collect();
        json!({"formatVersion":1,"recipeId":self.recipe_id,"title":self.title,
            "description":self.description,"kind":self.kind,"servings":self.servings,
            "prepMinutes":self.prep_minutes,"cookMinutes":self.cook_minutes,"notes":self.notes,
            "ingredients":ingredients,"steps":steps})
    }
    pub fn from_recipe(recipe: Recipe) -> Self {
        Self {
            format_version: 1,
            recipe_id: recipe.id,
            recipe_created_at: recipe.created_at,
            title: recipe.title,
            description: recipe.description,
            kind: recipe.kind,
            servings: recipe.servings,
            prep_minutes: recipe.prep_minutes,
            cook_minutes: recipe.cook_minutes,
            notes: recipe.notes,
            ingredients: recipe.ingredients,
            ingredient_names: recipe.ingredient_names,
            steps: recipe.steps,
        }
    }
    // Equality excludes capture labels, timestamps, lifecycle and optimistic
    // revision. Array order, IDs, exact quantities, NULLs and all step data count.
    pub(crate) fn content(&self) -> RecipeInput {
        let mut steps = self.steps.clone();
        for step in &mut steps {
            step.ingredient_line_ids.sort();
        }
        RecipeInput {
            id: self.recipe_id.clone(),
            expected_revision: None,
            title: self.title.clone(),
            description: self.description.clone(),
            kind: self.kind.clone(),
            servings: self.servings.clone(),
            prep_minutes: self.prep_minutes,
            cook_minutes: self.cook_minutes,
            notes: self.notes.clone(),
            ingredients: self.ingredients.clone(),
            steps,
        }
    }
    pub fn validate(&self) -> Result<(), AppError> {
        if self.format_version != 1 {
            return Err(AppError::new(
                "RECIPE_VERSION_UNSUPPORTED",
                "errors.recipeVersionUnsupported",
                true,
            ));
        }
        self.content().validate()?;
        let ids: std::collections::BTreeSet<_> =
            self.ingredients.iter().map(|i| &i.ingredient_id).collect();
        if self.ingredient_names.len() != ids.len()
            || self.recipe_created_at.is_empty()
            || self.recipe_created_at.len() > 100
            || ids
                .iter()
                .any(|id| !self.ingredient_names.contains_key(*id))
            || self
                .ingredient_names
                .values()
                .any(|n| n.is_empty() || n.chars().count() > 200 || n.contains('\0'))
        {
            return Err(invalid());
        }
        Ok(())
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecipeVersionSummary {
    pub recipe_id: String,
    pub version_number: i64,
    pub format_version: i64,
    pub source_revision: i64,
    pub provenance: String,
    pub restored_from_version: Option<i64>,
    pub captured_at: String,
    pub content_fingerprint: String,
    pub snapshot_checksum: String,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecipeVersion {
    pub summary: RecipeVersionSummary,
    pub snapshot: RecipeSnapshot,
}
