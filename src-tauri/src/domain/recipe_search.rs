use serde::{Deserialize, Serialize};

use super::{recipes::invalid, AppError};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RecipeSearchQuery {
    pub text: String,
    pub scope: String,
    pub kind: Option<String>,
    pub ingredient_id: Option<String>,
    #[serde(default)]
    pub favorite_only: bool,
    #[serde(default)]
    pub to_try_only: bool,
    #[serde(default)]
    pub collection_id: Option<String>,
    pub created_from: Option<String>,
    pub created_to: Option<String>,
    pub modified_from: Option<String>,
    pub modified_to: Option<String>,
    pub sort: String,
    pub offset: i64,
    pub limit: i64,
}

impl RecipeSearchQuery {
    pub fn validate(&self) -> Result<(), AppError> {
        fn date(value: &Option<String>) -> bool {
            value.as_ref().is_none_or(|v| {
                v.len() == 10
                    && v.as_bytes()[4] == b'-'
                    && v.as_bytes()[7] == b'-'
                    && v.chars()
                        .enumerate()
                        .all(|(i, c)| i == 4 || i == 7 || c.is_ascii_digit())
            })
        }
        if self.text.chars().count() > 200
            || self.text.contains('\0')
            || !["active", "archived", "trash"].contains(&self.scope.as_str())
            || (self.scope != "active"
                && (self.favorite_only || self.to_try_only || self.collection_id.is_some()))
            || self
                .kind
                .as_ref()
                .is_some_and(|k| !["food", "beverage"].contains(&k.as_str()))
            || ![
                "modified",
                "created",
                "title_asc",
                "title_desc",
                "relevance",
            ]
            .contains(&self.sort.as_str())
            || self.offset < 0
            || self.offset > 1_000_000
            || !(1..=50).contains(&self.limit)
            || !date(&self.created_from)
            || !date(&self.created_to)
            || !date(&self.modified_from)
            || !date(&self.modified_to)
        {
            return Err(invalid());
        }
        if let Some(id) = &self.ingredient_id {
            super::recipes::valid_id(id)?;
        }
        if let Some(id) = &self.collection_id {
            super::recipes::valid_id(id)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecipeSearchItem {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub kind: String,
    pub servings: String,
    pub created_at: String,
    pub updated_at: String,
    pub organization_revision: i64,
    pub favorite: bool,
    pub to_try: bool,
    pub rating: Option<i64>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecipeSearchPage {
    pub items: Vec<RecipeSearchItem>,
    pub total: i64,
    pub has_more: bool,
    pub suggestions: Vec<String>,
}
