use serde::{Deserialize, Serialize};
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecipeReference {
    pub id: String,
    pub title: String,
    pub revision: i64,
    pub status: String,
    pub relationship: String,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecipeLinks {
    pub graph_revision: i64,
    pub items: Vec<RecipeReference>,
    pub total: i64,
    pub has_more: bool,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PurgePreview {
    pub recipe_id: String,
    pub recipe_revision: i64,
    pub token: String,
    pub links: RecipeLinks,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ParentWrite {
    pub child_id: String,
    pub parent_id: Option<String>,
    pub expected_graph_revision: i64,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RelatedWrite {
    pub recipe_id: String,
    pub target_id: String,
    pub expected_graph_revision: i64,
    pub linked: bool,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReviewedPurge {
    pub recipe_id: String,
    pub expected_revision: i64,
    pub token: String,
    pub acknowledge_detach: bool,
}
