use super::{
    recipes::{decimal, search_name, valid_id},
    AppError,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

pub fn catalog_error(code: &str) -> AppError {
    AppError::new(code, "errors.catalog", true)
}
pub fn stable_id(namespace: &str, key: &str) -> String {
    let hash = Sha256::digest(format!("recipeatlas:{namespace}:{key}").as_bytes());
    let mut bytes = [0; 16];
    bytes.copy_from_slice(&hash[..16]);
    bytes[6] = (bytes[6] & 0x0f) | 0x80;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    uuid::Uuid::from_bytes(bytes).to_string()
}
pub fn name_key(locale: &str, name: &str) -> String {
    if locale == "tr" {
        search_name(name)
    } else {
        use unicode_normalization::UnicodeNormalization;
        name.nfc()
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .to_lowercase()
    }
}
pub fn text(s: &str, max: usize) -> bool {
    !s.trim().is_empty() && s.chars().count() <= max && !s.contains('\0')
}
pub fn key(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 120
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"-_.".contains(&b))
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SourceSnapshot {
    pub id: String,
    pub version: String,
    pub name: String,
    pub url: String,
    pub license: String,
    pub attribution: String,
    pub retrieved_at: String,
    pub artifact_sha256: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Artifact {
    pub file: String,
    pub sha256: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SeedManifest {
    pub format_version: i64,
    pub dataset: String,
    pub version: i64,
    pub purpose: String,
    pub artifact: Artifact,
    pub evidence: BTreeMap<String, Artifact>,
    pub sources: Vec<SourceSnapshot>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CategorySeed {
    pub key: String,
    pub parent: Option<String>,
    pub tr: String,
    pub en: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LocalizedName {
    pub locale: String,
    pub name: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Provenance {
    pub source_id: String,
    pub source_version: String,
    pub external_id: String,
    pub description: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Relation {
    pub key: String,
    pub kind: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Observation {
    pub kind: String,
    pub code: String,
    pub value: String,
    pub unit: Option<String>,
    pub basis: String,
    pub source_id: String,
    pub source_version: String,
    pub external_id: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CatalogEntry {
    pub id: String,
    pub key: String,
    pub canonical_name: String,
    pub ingredient_type: String,
    pub names: BTreeMap<String, String>,
    pub aliases: Vec<LocalizedName>,
    pub categories: Vec<String>,
    pub preferred_unit: Option<String>,
    pub dimensions: Vec<String>,
    pub relations: Vec<Relation>,
    pub provenance: Vec<Provenance>,
    pub observations: Vec<Observation>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SeedData {
    pub categories: Vec<CategorySeed>,
    pub ingredients: Vec<CatalogEntry>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ImportReport {
    pub dataset: String,
    pub version: i64,
    pub purpose: String,
    pub unchanged: bool,
    pub validated: usize,
    pub inserted: usize,
    pub updated: usize,
    pub available: usize,
    pub collisions: usize,
    pub category_coverage: BTreeMap<String, usize>,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogStatus {
    pub definitions: i64,
    pub validation_definitions: i64,
    pub production_definitions: i64,
    pub pending_collisions: i64,
}

pub struct VerifiedSeed {
    pub manifest: SeedManifest,
    pub data: SeedData,
    pub manifest_hash: String,
    pub artifact_hash: String,
}
impl VerifiedSeed {
    pub fn validate(&self) -> Result<(), AppError> {
        use std::collections::HashSet;
        let m = &self.manifest;
        let fail = || catalog_error("CATALOG_INVALID");
        if m.format_version != 1
            || !key(&m.dataset)
            || m.version < 1
            || !["validation", "production"].contains(&m.purpose.as_str())
            || self.data.ingredients.is_empty()
            || self.data.ingredients.len() > 50000
            || self.data.categories.len() > 2000
        {
            return Err(fail());
        }
        let mut sources = HashSet::new();
        for s in &m.sources {
            if !sources.insert((&s.id, &s.version))
                || !key(&s.id)
                || !text(&s.version, 100)
                || !text(&s.name, 200)
                || !s.url.starts_with("https://")
                || !text(&s.attribution, 2000)
                || !text(&s.retrieved_at, 40)
            {
                return Err(fail());
            }
        }
        let mut categories = BTreeMap::new();
        for c in &self.data.categories {
            if !key(&c.key)
                || !text(&c.tr, 200)
                || !text(&c.en, 200)
                || categories
                    .insert(c.key.as_str(), c.parent.as_deref())
                    .is_some()
            {
                return Err(fail());
            }
        }
        for c in &self.data.categories {
            let mut seen = HashSet::from([c.key.as_str()]);
            let mut parent = c.parent.as_deref();
            while let Some(p) = parent {
                if !seen.insert(p) {
                    return Err(catalog_error("CATALOG_CATEGORY_CYCLE"));
                }
                parent = *categories.get(p).ok_or_else(fail)?;
            }
        }
        let mut ids = HashSet::new();
        let mut keys = HashSet::new();
        let mut names = HashSet::new();
        let mut provenance = HashSet::new();
        for i in &self.data.ingredients {
            valid_id(&i.id)?;
            if !key(&i.key)
                || i.id != stable_id("ingredient", &i.key)
                || !ids.insert(&i.id)
                || !keys.insert(&i.key)
                || !text(&i.canonical_name, 200)
                || !["food", "beverage", "alcohol", "garnish", "other"]
                    .contains(&i.ingredient_type.as_str())
                || i.names.len() != 2
                || i.provenance.is_empty()
            {
                return Err(fail());
            }
            for locale in ["tr", "en"] {
                let name = i.names.get(locale).ok_or_else(fail)?;
                if !text(name, 200) || !names.insert((locale, name_key(locale, name))) {
                    return Err(catalog_error("CATALOG_DUPLICATE"));
                }
            }
            let mut aliases = HashSet::new();
            for a in &i.aliases {
                if !["tr", "en"].contains(&a.locale.as_str())
                    || !text(&a.name, 200)
                    || !aliases.insert((&a.locale, name_key(&a.locale, &a.name)))
                {
                    return Err(fail());
                }
            }
            let mut memberships = HashSet::new();
            for c in &i.categories {
                if !categories.contains_key(c.as_str()) || !memberships.insert(c) {
                    return Err(fail());
                }
            }
            if memberships.is_empty() {
                return Err(fail());
            }
            let mut dims = HashSet::new();
            for d in &i.dimensions {
                if !["mass", "volume", "count"].contains(&d.as_str()) || !dims.insert(d) {
                    return Err(fail());
                }
            }
            for p in &i.provenance {
                if !sources.contains(&(&p.source_id, &p.source_version))
                    || !text(&p.external_id, 200)
                    || !text(&p.description, 1000)
                    || !provenance.insert((&p.source_id, &p.source_version, &p.external_id))
                {
                    return Err(fail());
                }
            }
            let mut observations = HashSet::new();
            for o in &i.observations {
                if !observations.insert((&o.kind, &o.code, &o.basis))
                    || !text(&o.code, 100)
                    || !text(&o.basis, 500)
                    || !i.provenance.iter().any(|p| {
                        p.source_id == o.source_id
                            && p.source_version == o.source_version
                            && p.external_id == o.external_id
                    })
                {
                    return Err(fail());
                }
                let zero = o.value == "0"
                    || (o.value.starts_with("0.")
                        && o.value.len() <= 8
                        && o.value[2..].bytes().all(|b| b == b'0')
                        && o.value.len() > 2);
                let valid = match o.kind.as_str() {
                    "density" => decimal(&o.value) && o.unit.as_deref() == Some("g/mL"),
                    "nutrient" => {
                        (decimal(&o.value) || zero) && o.unit.as_ref().is_some_and(|u| text(u, 30))
                    }
                    "abv" => {
                        (decimal(&o.value) || zero)
                            && o.unit.as_deref() == Some("%")
                            && o.value
                                .split('.')
                                .next()
                                .and_then(|v| v.parse::<u32>().ok())
                                .is_some_and(|n| {
                                    n < 100
                                        || n == 100
                                            && o.value
                                                .split('.')
                                                .nth(1)
                                                .is_none_or(|v| v.bytes().all(|b| b == b'0'))
                                })
                    }
                    "allergen" => {
                        ["present", "absent", "unknown"].contains(&o.value.as_str())
                            && o.unit.is_none()
                    }
                    "dietary" => {
                        ["yes", "no", "unknown"].contains(&o.value.as_str()) && o.unit.is_none()
                    }
                    _ => false,
                };
                if !valid {
                    return Err(fail());
                }
            }
        }
        for i in &self.data.ingredients {
            let mut edges = HashSet::new();
            for r in &i.relations {
                if r.key == i.key
                    || !keys.contains(&r.key)
                    || !["form_of", "related"].contains(&r.kind.as_str())
                    || !edges.insert((&r.key, &r.kind))
                {
                    return Err(fail());
                }
            }
        }
        // A form hierarchy must be acyclic; ordinary related edges may be symmetric.
        let mut incoming: BTreeMap<&str, usize> = self
            .data
            .ingredients
            .iter()
            .map(|i| (i.key.as_str(), 0))
            .collect();
        let mut children: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
        for i in &self.data.ingredients {
            for r in i.relations.iter().filter(|r| r.kind == "form_of") {
                *incoming.get_mut(i.key.as_str()).ok_or_else(fail)? += 1;
                children.entry(&r.key).or_default().push(&i.key);
            }
        }
        let mut ready: Vec<_> = incoming
            .iter()
            .filter(|(_, n)| **n == 0)
            .map(|(k, _)| *k)
            .collect();
        let mut visited = 0;
        while let Some(parent) = ready.pop() {
            visited += 1;
            for child in children.get(parent).into_iter().flatten() {
                let n = incoming.get_mut(child).ok_or_else(fail)?;
                *n -= 1;
                if *n == 0 {
                    ready.push(child);
                }
            }
        }
        if visited != incoming.len() {
            return Err(catalog_error("CATALOG_RELATION_CYCLE"));
        }
        Ok(())
    }
}
