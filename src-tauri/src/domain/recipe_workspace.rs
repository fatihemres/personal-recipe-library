use serde::{Deserialize, Serialize};

use super::{
    recipe_search::RecipeSearchQuery,
    recipes::{invalid, valid_id},
    AppError,
};

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PrivateNote {
    pub recipe_id: String,
    pub format_version: i64,
    pub markdown: String,
    pub revision: i64,
    pub updated_at: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PrivateNoteWrite {
    pub recipe_id: String,
    pub expected_revision: i64,
    pub markdown: String,
}
impl PrivateNoteWrite {
    pub fn validate(&self) -> Result<(), AppError> {
        valid_id(&self.recipe_id)?;
        if self.expected_revision < 0
            || self.markdown.chars().count() > 50_000
            || self.markdown.contains('\0')
        {
            return Err(invalid());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedFilter {
    pub id: String,
    pub title: String,
    pub revision: i64,
    pub format_version: i64,
    pub query: Option<RecipeSearchQuery>,
    pub available: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SavedFilterWrite {
    pub id: String,
    pub expected_revision: Option<i64>,
    pub title: String,
    pub query: RecipeSearchQuery,
}
impl SavedFilterWrite {
    pub fn validate(&self) -> Result<(), AppError> {
        valid_id(&self.id)?;
        self.query.validate()?;
        if self.expected_revision.is_some_and(|value| value < 1)
            || self.title.trim().is_empty()
            || self.title.chars().count() > 100
            || self.title.contains('\0')
        {
            return Err(invalid());
        }
        if self.query.offset != 0 || self.query.limit != 24 {
            return Err(invalid());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Dashboard {
    pub active_total: i64,
    pub food_count: i64,
    pub beverage_count: i64,
    pub favorite_count: i64,
    pub recent_created: Vec<super::recipe_search::RecipeSearchItem>,
    pub recent_modified: Vec<super::recipe_search::RecipeSearchItem>,
    pub collections: Vec<super::recipe_organization::RecipeCollection>,
}
