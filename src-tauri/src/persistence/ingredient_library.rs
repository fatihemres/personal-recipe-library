use super::Database;
use crate::domain::{
    catalog::{name_key, LocalizedName},
    ingredient_library::*,
    recipes::{search_name, valid_id},
    AppError,
};
use rusqlite::{params, Connection, OptionalExtension, Row};

// Canonical definitions remain discoverable even when a personal-name collision
// prevents materialization. recipe_id is absent in that case; never merge IDs.
const BASE: &str = "WITH RECURSIVE
 library(id,origin,catalog_id,recipe_id,name,en,search_name,notes,preferred_unit,revision,ingredient_type) AS (
 SELECT c.id,'catalog',c.id,i.id,coalesce(i.name,n.name),e.name,coalesce(i.search_name,n.search_key),i.notes,coalesce(i.preferred_unit,c.preferred_unit),i.revision,c.ingredient_type
 FROM catalog_ingredients c JOIN catalog_names n ON n.ingredient_id=c.id AND n.locale='tr'
 LEFT JOIN catalog_names e ON e.ingredient_id=c.id AND e.locale='en'
 LEFT JOIN ingredients i ON i.id=c.id AND i.origin='catalog'
 UNION ALL
 SELECT i.id,'personal',i.catalog_id,i.id,i.name,e.name,i.search_name,i.notes,i.preferred_unit,i.revision,c.ingredient_type
 FROM ingredients i LEFT JOIN catalog_ingredients c ON c.id=i.catalog_id
 LEFT JOIN catalog_names e ON e.ingredient_id=c.id AND e.locale='en' WHERE i.origin='personal'
 ), members(origin,id,category_id) AS (
 SELECT 'catalog',ingredient_id,category_id FROM catalog_category_memberships
 UNION SELECT 'personal',ingredient_id,category_id FROM personal_category_memberships
 UNION SELECT 'personal',i.id,m.category_id FROM ingredients i JOIN catalog_category_memberships m ON m.ingredient_id=i.catalog_id WHERE i.origin='personal'
 ), tree(root,id) AS (
 SELECT id,id FROM ingredient_categories UNION SELECT t.root,c.id FROM tree t JOIN ingredient_categories c ON c.parent_id=t.id
 ), searched AS (
 SELECT * FROM library l WHERE instr(l.search_name,?1)>0
 OR EXISTS(SELECT 1 FROM catalog_names n WHERE n.ingredient_id=l.catalog_id AND instr(n.search_key,CASE WHEN n.locale='tr' THEN ?1 ELSE ?2 END)>0)
 OR EXISTS(SELECT 1 FROM catalog_aliases a WHERE a.ingredient_id=l.catalog_id AND instr(a.search_key,CASE WHEN a.locale='tr' THEN ?1 ELSE ?2 END)>0)
 OR EXISTS(SELECT 1 FROM ingredient_personal_aliases a WHERE a.catalog_id=l.catalog_id AND instr(a.search_key,?1)>0)
 )";
const FILTER: &str = "(?3='all' OR l.origin=?3) AND (?4 IS NULL OR EXISTS(SELECT 1 FROM members m JOIN tree t ON t.id=m.category_id WHERE m.id=l.id AND m.origin=l.origin AND t.root=?4)) AND (?8=0 OR NOT EXISTS(SELECT 1 FROM members m WHERE m.id=l.id AND m.origin=l.origin)) AND (?5 IS NULL OR (?5='favorites' AND EXISTS(SELECT 1 FROM favorites f WHERE f.id=l.id AND f.origin=l.origin)) OR (?5='recent' AND EXISTS(SELECT 1 FROM usage u WHERE u.id=l.recipe_id)))";
const CONTEXT: &str = "favorites(origin,id) AS (SELECT 'catalog',catalog_id FROM catalog_ingredient_favorites UNION ALL SELECT 'personal',ingredient_id FROM personal_ingredient_favorites), usage(id,last_used_at) AS (SELECT ri.ingredient_id,max(r.updated_at) FROM recipe_ingredients ri JOIN recipes r ON r.id=ri.recipe_id WHERE r.deleted_at IS NULL GROUP BY ri.ingredient_id)";

fn category(row: &Row<'_>, start: usize) -> rusqlite::Result<Category> {
    Ok(Category {
        id: row.get(start)?,
        parent_id: row.get(start + 1)?,
        key: row.get(start + 2)?,
        tr: row.get(start + 3)?,
        en: row.get(start + 4)?,
    })
}
fn item(row: &Row<'_>) -> rusqlite::Result<LibraryItem> {
    Ok(LibraryItem {
        id: row.get(0)?,
        origin: row.get(1)?,
        recipe_id: row.get(2)?,
        name: row.get(3)?,
        english_name: row.get(4)?,
        notes: row.get(5)?,
        preferred_unit: row.get(6)?,
        revision: row.get(7)?,
        ingredient_type: row.get(8)?,
        categories: Vec::new(),
        favorite: row.get(9)?,
        last_used_at: row.get(10)?,
    })
}
fn memberships(conn: &Connection, origin: &str, id: &str) -> Result<Vec<Category>, AppError> {
    let sql = format!("{BASE} SELECT c.id,c.parent_id,c.stable_key,c.name_tr,c.name_en FROM ingredient_categories c JOIN members m ON m.category_id=c.id WHERE m.id=?3 AND m.origin=?4 ORDER BY (SELECT count(*) FROM tree t WHERE t.id=c.id) DESC,c.name_tr,c.id");
    let mut statement = conn.prepare(&sql)?;
    let rows = statement
        .query_map(params!["", "", id, origin], |r| category(r, 0))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}
impl Database {
    pub fn ingredient_library(&self, query: LibraryQuery) -> Result<LibraryPage, AppError> {
        if query.search.chars().count() > 200
            || query.search.contains('\0')
            || !["all", "catalog", "personal"].contains(&query.origin.as_str())
            || !(1..=100).contains(&query.limit)
            || !(0..=1_000_000).contains(&query.offset)
            || query
                .view
                .as_deref()
                .is_some_and(|v| !["favorites", "recent"].contains(&v))
            || (query.uncategorized && query.category_id.is_some())
        {
            return Err(AppError::new("INVALID_INPUT", "errors.validation", true));
        }
        if let Some(id) = &query.category_id {
            valid_id(id)?;
        }
        let tx = self.conn.unchecked_transaction()?;
        if let Some(id) = &query.category_id {
            if !tx.query_row(
                "SELECT EXISTS(SELECT 1 FROM ingredient_categories WHERE id=?1)",
                [id],
                |r| r.get::<_, bool>(0),
            )? {
                return Err(AppError::new("NOT_FOUND", "errors.notFound", true));
            }
        }
        let tr = search_name(&query.search);
        let en = name_key("en", &query.search);
        let (catalog_count,personal_count)=tx.query_row("SELECT (SELECT count(*) FROM catalog_ingredients),(SELECT count(*) FROM ingredients WHERE origin='personal')",[],|r|Ok((r.get(0)?,r.get(1)?)))?;
        let total = tx.query_row(
            &format!("{BASE}, {CONTEXT} SELECT count(*) FROM searched l WHERE {FILTER}"),
            params![
                tr,
                en,
                query.origin,
                query.category_id,
                query.view,
                query.limit,
                query.offset,
                query.uncategorized
            ],
            |r| r.get(0),
        )?;
        let mut items = {
            let order = if query.view.as_deref() == Some("recent") {
                "u.last_used_at DESC,l.search_name,l.origin,l.id"
            } else {
                "l.search_name,l.origin,l.id"
            };
            let mut s=tx.prepare(&format!("{BASE}, {CONTEXT} SELECT l.id,l.origin,l.recipe_id,l.name,l.en,l.notes,l.preferred_unit,l.revision,l.ingredient_type, EXISTS(SELECT 1 FROM favorites f WHERE f.id=l.id AND f.origin=l.origin),u.last_used_at FROM searched l LEFT JOIN usage u ON u.id=l.recipe_id WHERE {FILTER} ORDER BY {order} LIMIT ?6 OFFSET ?7"))?;
            let rows = s
                .query_map(
                    params![
                        tr,
                        en,
                        query.origin,
                        query.category_id,
                        query.view,
                        query.limit,
                        query.offset,
                        query.uncategorized
                    ],
                    item,
                )?
                .collect::<Result<Vec<_>, _>>()?;
            rows
        };
        for i in &mut items {
            i.categories = memberships(&tx, &i.origin, &i.id)?;
        }
        // Facets honor search and origin, but ignore the selected category so
        // another branch remains reachable. Descendants count each record once.
        let categories = {
            let sql=format!("{BASE}, {CONTEXT}, matches AS (SELECT DISTINCT t.root,l.origin,l.id FROM searched l JOIN members m ON m.id=l.id AND m.origin=l.origin JOIN tree t ON t.id=m.category_id WHERE (?3='all' OR l.origin=?3) AND (?5 IS NULL OR (?5='favorites' AND EXISTS(SELECT 1 FROM favorites f WHERE f.id=l.id AND f.origin=l.origin)) OR (?5='recent' AND EXISTS(SELECT 1 FROM usage u WHERE u.id=l.recipe_id)))) SELECT c.id,c.parent_id,c.stable_key,c.name_tr,c.name_en,count(m.id) FROM ingredient_categories c LEFT JOIN matches m ON m.root=c.id GROUP BY c.id ORDER BY (SELECT count(*) FROM tree t WHERE t.id=c.id) DESC,c.name_tr,c.id");
            let mut s = tx.prepare(&sql)?;
            let rows = s
                .query_map(
                    params![tr, en, query.origin, query.category_id, query.view],
                    |r| {
                        Ok(CategoryCount {
                            category: category(r, 0)?,
                            count: r.get(5)?,
                        })
                    },
                )?
                .collect::<Result<Vec<_>, _>>()?;
            rows
        };
        tx.commit()?;
        Ok(LibraryPage {
            items,
            total,
            catalog_count,
            personal_count,
            categories,
            offset: query.offset,
            limit: query.limit,
        })
    }
    pub fn ingredient_detail(&self, id: &str, origin: &str) -> Result<LibraryDetail, AppError> {
        valid_id(id)?;
        if !["catalog", "personal"].contains(&origin) {
            return Err(AppError::new("INVALID_INPUT", "errors.validation", true));
        }
        let tx = self.conn.unchecked_transaction()?;
        let mut value=tx.query_row(&format!("{BASE}, {CONTEXT} SELECT l.id,l.origin,l.recipe_id,l.name,l.en,l.notes,l.preferred_unit,l.revision,l.ingredient_type,EXISTS(SELECT 1 FROM favorites f WHERE f.id=l.id AND f.origin=l.origin),u.last_used_at FROM library l LEFT JOIN usage u ON u.id=l.recipe_id WHERE l.id=?3 AND l.origin=?4"),params!["","",id,origin],item).map_err(|e| match e { rusqlite::Error::QueryReturnedNoRows=>AppError::new("NOT_FOUND","errors.notFound",true), _=>e.into() })?;
        value.categories = memberships(&tx, origin, id)?;
        let catalog_id: Option<String> = tx.query_row(
            &format!("{BASE} SELECT catalog_id FROM library WHERE id=?3 AND origin=?4"),
            params!["", "", id, origin],
            |r| r.get(0),
        )?;
        let mut detail = LibraryDetail {
            item: value,
            canonical_tr: None,
            aliases: vec![],
            dimensions: vec![],
            sources: vec![],
            metadata: vec![],
            catalog_version: None,
            personal_alias: None,
        };
        if let Some(catalog_id) = catalog_id {
            detail.personal_alias = tx
                .query_row(
                    "SELECT alias FROM ingredient_personal_aliases WHERE catalog_id=?1",
                    [&catalog_id],
                    |r| r.get(0),
                )
                .optional()?;
            detail.canonical_tr = Some(tx.query_row(
                "SELECT name FROM catalog_names WHERE ingredient_id=?1 AND locale='tr'",
                [&catalog_id],
                |r| r.get(0),
            )?);
            detail.catalog_version = Some(tx.query_row(
                "SELECT catalog_version FROM catalog_ingredients WHERE id=?1",
                [&catalog_id],
                |r| r.get(0),
            )?);
            let mut s=tx.prepare("SELECT locale,name FROM catalog_aliases WHERE ingredient_id=?1 ORDER BY locale,name")?;
            detail.aliases = s
                .query_map([&catalog_id], |r| {
                    Ok(LocalizedName {
                        locale: r.get(0)?,
                        name: r.get(1)?,
                    })
                })?
                .collect::<Result<_, _>>()?;
            let mut s=tx.prepare("SELECT dimension FROM catalog_dimensions WHERE ingredient_id=?1 ORDER BY dimension")?;
            detail.dimensions = s
                .query_map([&catalog_id], |r| r.get(0))?
                .collect::<Result<_, _>>()?;
            let mut s=tx.prepare("SELECT s.name,p.source_id,p.source_version,p.external_id,s.url,s.license,s.attribution,p.source_description FROM catalog_provenance p JOIN catalog_sources s ON s.id=p.source_id AND s.version=p.source_version WHERE p.ingredient_id=?1 ORDER BY p.source_id,p.source_version")?;
            detail.sources = s
                .query_map([&catalog_id], |r| {
                    Ok(SourceFact {
                        name: r.get(0)?,
                        source_id: r.get(1)?,
                        version: r.get(2)?,
                        external_id: r.get(3)?,
                        url: r.get(4)?,
                        license: r.get(5)?,
                        attribution: r.get(6)?,
                        description: r.get(7)?,
                    })
                })?
                .collect::<Result<_, _>>()?;
            let mut s=tx.prepare("SELECT kind,code,value,unit,basis FROM catalog_observations WHERE ingredient_id=?1 ORDER BY kind,code,basis")?;
            detail.metadata = s
                .query_map([&catalog_id], |r| {
                    Ok(FactualMetadata {
                        kind: r.get(0)?,
                        code: r.get(1)?,
                        value: r.get(2)?,
                        unit: r.get(3)?,
                        basis: r.get(4)?,
                    })
                })?
                .collect::<Result<_, _>>()?;
        }
        tx.commit()?;
        Ok(detail)
    }
}

impl Database {
    pub fn set_ingredient_favorite(
        &mut self,
        id: &str,
        origin: &str,
        favorite: bool,
    ) -> Result<bool, AppError> {
        valid_id(id)?;
        let (table, column, exists) = match origin {
            "catalog" => (
                "catalog_ingredient_favorites",
                "catalog_id",
                "catalog_ingredients",
            ),
            "personal" => (
                "personal_ingredient_favorites",
                "ingredient_id",
                "ingredients",
            ),
            _ => return Err(AppError::new("INVALID_INPUT", "errors.validation", true)),
        };
        let tx = self
            .conn
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        let available: bool = tx.query_row(
            &format!(
                "SELECT EXISTS(SELECT 1 FROM {exists} WHERE id=?1{} )",
                if origin == "personal" {
                    " AND origin='personal'"
                } else {
                    ""
                }
            ),
            [id],
            |r| r.get(0),
        )?;
        if !available {
            return Err(AppError::new("NOT_FOUND", "errors.notFound", true));
        }
        if favorite {
            tx.execute(
                &format!("INSERT OR IGNORE INTO {table}({column}) VALUES(?1)"),
                [id],
            )?;
        } else {
            tx.execute(&format!("DELETE FROM {table} WHERE {column}=?1"), [id])?;
        }
        tx.commit()?;
        Ok(favorite)
    }
    pub fn set_catalog_presentation(
        &mut self,
        input: CatalogPresentationInput,
    ) -> Result<LibraryDetail, AppError> {
        valid_id(&input.id)?;
        let valid_text = |v: &str, max| !v.contains('\0') && v.chars().count() <= max;
        if input.revision < 1
            || input
                .name
                .as_deref()
                .is_some_and(|v| !valid_text(v, 200) || v.trim().is_empty())
            || input
                .alias
                .as_deref()
                .is_some_and(|v| !valid_text(v, 200) || v.trim().is_empty())
            || input.notes.as_deref().is_some_and(|v| !valid_text(v, 2000))
        {
            return Err(AppError::new("INVALID_INPUT", "errors.validation", true));
        }
        let tx = self
            .conn
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        let (canonical_name,canonical_unit):(String,Option<String>)=tx.query_row("SELECT n.name,c.preferred_unit FROM catalog_ingredients c JOIN catalog_names n ON n.ingredient_id=c.id AND n.locale='tr' WHERE c.id=?1",[&input.id],|r|Ok((r.get(0)?,r.get(1)?))).optional()?.ok_or_else(||AppError::new("NOT_FOUND","errors.notFound",true))?;
        let current:Option<(i64,String,Option<String>,Option<String>)>=tx.query_row("SELECT revision,name,notes,preferred_unit FROM ingredients WHERE id=?1 AND origin='catalog'",[&input.id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).optional()?;
        let (revision, old_name, old_notes, old_unit) =
            current.ok_or_else(|| AppError::new("CATALOG_COLLISION", "errors.catalog", true))?;
        if revision != input.revision {
            return Err(crate::domain::recipes::conflict());
        }
        let name = input
            .name
            .as_deref()
            .map(str::trim)
            .unwrap_or(&canonical_name);
        let unit = input
            .preferred_unit
            .as_deref()
            .or(canonical_unit.as_deref());
        if let Some(code) = unit {
            let valid:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM catalog_dimensions d JOIN units u ON u.dimension=d.dimension WHERE d.ingredient_id=?1 AND u.code=?2)",params![input.id,code],|r|r.get(0))?;
            if !valid {
                return Err(AppError::new("INVALID_INPUT", "errors.validation", true));
            }
        }
        let normalized_name = search_name(name);
        let collision: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM ingredients WHERE id<>?1 AND search_name=?2)",
            params![input.id, normalized_name],
            |r| r.get(0),
        )?;
        if collision {
            return Err(AppError::new(
                "INGREDIENT_DUPLICATE",
                "errors.ingredientDuplicate",
                true,
            ));
        }
        let alias = input.alias.as_deref().map(str::trim);
        if let Some(value) = alias {
            let alias_tr = search_name(value);
            let alias_en = name_key("en", value);
            let collision:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM ingredients WHERE id<>?1 AND search_name=?2) OR EXISTS(SELECT 1 FROM catalog_names WHERE ingredient_id<>?1 AND search_key=CASE WHEN locale='tr' THEN ?2 ELSE ?3 END) OR EXISTS(SELECT 1 FROM catalog_aliases WHERE ingredient_id<>?1 AND search_key=CASE WHEN locale='tr' THEN ?2 ELSE ?3 END) OR EXISTS(SELECT 1 FROM ingredient_personal_aliases WHERE catalog_id<>?1 AND search_key=?2)",params![input.id,alias_tr,alias_en],|r|r.get(0))?;
            if collision {
                return Err(AppError::new(
                    "INGREDIENT_DUPLICATE",
                    "errors.ingredientDuplicate",
                    true,
                ));
            }
        }
        let changed = old_name != name || old_notes != input.notes || old_unit.as_deref() != unit;
        let old_alias: Option<String> = tx
            .query_row(
                "SELECT alias FROM ingredient_personal_aliases WHERE catalog_id=?1",
                [&input.id],
                |r| r.get(0),
            )
            .optional()?;
        if changed || old_alias.as_deref() != alias {
            tx.execute("UPDATE ingredients SET name=?2,search_name=?3,notes=?4,preferred_unit=?5,revision=revision+1,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1 AND revision=?6",params![input.id,name,normalized_name,input.notes,unit,input.revision])?;
            for (field, active) in [
                ("name", name != canonical_name),
                ("notes", input.notes.is_some()),
                ("preferred_unit", unit != canonical_unit.as_deref()),
            ] {
                if active {
                    tx.execute("INSERT INTO ingredient_customizations(ingredient_id,field) VALUES(?1,?2) ON CONFLICT(ingredient_id,field) DO UPDATE SET changed_at=strftime('%Y-%m-%dT%H:%M:%fZ','now')",params![input.id,field])?;
                } else {
                    tx.execute(
                        "DELETE FROM ingredient_customizations WHERE ingredient_id=?1 AND field=?2",
                        params![input.id, field],
                    )?;
                }
            }
            if let Some(value) = alias {
                tx.execute("INSERT INTO ingredient_personal_aliases(catalog_id,alias,search_key) VALUES(?1,?2,?3) ON CONFLICT(catalog_id) DO UPDATE SET alias=excluded.alias,search_key=excluded.search_key,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now')",params![input.id,value,search_name(value)])?;
            } else {
                tx.execute(
                    "DELETE FROM ingredient_personal_aliases WHERE catalog_id=?1",
                    [&input.id],
                )?;
            }
        }
        tx.commit()?;
        self.ingredient_detail(&input.id, "catalog")
    }
    pub fn ingredient_candidates(&self, name: &str) -> Result<Vec<IngredientCandidate>, AppError> {
        if name.chars().count() > 200 || name.contains('\0') {
            return Err(AppError::new("INVALID_INPUT", "errors.validation", true));
        }
        let tr = search_name(name);
        let en = name_key("en", name);
        if tr.chars().count() < 2 {
            return Ok(vec![]);
        }
        let mut s=self.conn.prepare(r#"
            WITH candidates AS (
              SELECT i.id,i.origin,i.name,
                CASE WHEN i.search_name=?1 THEN 'exact_name' ELSE 'ambiguous' END reason,
                CASE WHEN i.search_name=?1 THEN 0 WHEN instr(i.search_name,?1)=1 THEN 3 ELSE 4 END rank
              FROM ingredients i WHERE i.origin='personal' AND instr(i.search_name,?1)>0
              UNION ALL
              SELECT c.id,'catalog',coalesce(i.name,n.name),
                CASE WHEN n.search_key=?1 OR e.search_key=?2 OR i.search_name=?1 THEN 'exact_name'
                  WHEN EXISTS(SELECT 1 FROM catalog_aliases a WHERE a.ingredient_id=c.id AND a.search_key=CASE WHEN a.locale='tr' THEN ?1 ELSE ?2 END)
                    OR EXISTS(SELECT 1 FROM ingredient_personal_aliases a WHERE a.catalog_id=c.id AND a.search_key=?1) THEN 'possible_synonym'
                  ELSE 'ambiguous' END,
                CASE WHEN n.search_key=?1 OR e.search_key=?2 OR i.search_name=?1 THEN 0
                  WHEN EXISTS(SELECT 1 FROM catalog_aliases a WHERE a.ingredient_id=c.id AND a.search_key=CASE WHEN a.locale='tr' THEN ?1 ELSE ?2 END)
                    OR EXISTS(SELECT 1 FROM ingredient_personal_aliases a WHERE a.catalog_id=c.id AND a.search_key=?1) THEN 1
                  WHEN instr(n.search_key,?1)=1 OR instr(e.search_key,?2)=1 THEN 3 ELSE 4 END
              FROM catalog_ingredients c JOIN catalog_names n ON n.ingredient_id=c.id AND n.locale='tr'
                LEFT JOIN catalog_names e ON e.ingredient_id=c.id AND e.locale='en'
                LEFT JOIN ingredients i ON i.id=c.id AND i.origin='catalog'
              WHERE instr(n.search_key,?1)>0 OR instr(e.search_key,?2)>0 OR instr(i.search_name,?1)>0
                OR EXISTS(SELECT 1 FROM catalog_aliases a WHERE a.ingredient_id=c.id AND instr(a.search_key,CASE WHEN a.locale='tr' THEN ?1 ELSE ?2 END)>0)
                OR EXISTS(SELECT 1 FROM ingredient_personal_aliases a WHERE a.catalog_id=c.id AND instr(a.search_key,?1)>0)
            ) SELECT id,origin,name,reason FROM candidates ORDER BY rank,name,id LIMIT 6
        "#)?;
        let rows = s
            .query_map(params![tr, en], |r| {
                Ok(IngredientCandidate {
                    id: r.get(0)?,
                    origin: r.get(1)?,
                    name: r.get(2)?,
                    reason: r.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }
}
