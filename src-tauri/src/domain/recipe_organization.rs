use serde::{Deserialize, Serialize};

use super::{
    recipes::{invalid, valid_id},
    AppError,
};

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecipeOrganization {
    pub recipe_id: String,
    pub revision: i64,
    pub favorite: bool,
    pub to_try: bool,
    pub rating: Option<i64>,
    pub collection_ids: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OrganizationWrite {
    pub recipe_id: String,
    pub expected_revision: i64,
    pub favorite: bool,
    pub to_try: bool,
    pub rating: Option<i64>,
}
impl OrganizationWrite {
    pub fn validate(&self) -> Result<(), AppError> {
        valid_id(&self.recipe_id)?;
        if self.expected_revision < 1 || self.rating.is_some_and(|r| !(1..=5).contains(&r)) {
            return Err(invalid());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecipeCollection {
    pub id: String,
    pub title: String,
    pub revision: i64,
    pub active_count: i64,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CollectionWrite {
    pub id: String,
    pub expected_revision: Option<i64>,
    pub title: String,
}
impl CollectionWrite {
    pub fn validate(&self) -> Result<(), AppError> {
        valid_id(&self.id)?;
        if self.expected_revision.is_some_and(|r| r < 1)
            || self.title.trim().is_empty()
            || self.title.chars().count() > 100
            || self.title.contains('\0')
        {
            return Err(invalid());
        }
        Ok(())
    }
}
