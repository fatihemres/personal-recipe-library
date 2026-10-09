use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LibraryQuery {
    pub search: String,
    pub origin: String,
    pub category_id: Option<String>,
    pub offset: i64,
    pub limit: i64,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Category {
    pub id: String,
    pub parent_id: Option<String>,
    pub key: String,
    pub tr: String,
    pub en: String,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoryCount {
    pub category: Category,
    pub count: i64,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryItem {
    pub id: String,
    pub origin: String,
    pub recipe_id: Option<String>,
    pub name: String,
    pub english_name: Option<String>,
    pub notes: Option<String>,
    pub preferred_unit: Option<String>,
    pub revision: Option<i64>,
    pub ingredient_type: Option<String>,
    pub categories: Vec<Category>,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryPage {
    pub items: Vec<LibraryItem>,
    pub total: i64,
    pub catalog_count: i64,
    pub personal_count: i64,
    pub categories: Vec<CategoryCount>,
    pub offset: i64,
    pub limit: i64,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceFact {
    pub name: String,
    pub source_id: String,
    pub version: String,
    pub external_id: String,
    pub url: String,
    pub license: String,
    pub attribution: String,
    pub description: String,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FactualMetadata {
    pub kind: String,
    pub code: String,
    pub value: String,
    pub unit: Option<String>,
    pub basis: String,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryDetail {
    pub item: LibraryItem,
    pub canonical_tr: Option<String>,
    pub aliases: Vec<super::catalog::LocalizedName>,
    pub dimensions: Vec<String>,
    pub sources: Vec<SourceFact>,
    pub metadata: Vec<FactualMetadata>,
    pub catalog_version: Option<i64>,
}
