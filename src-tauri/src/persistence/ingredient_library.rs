use super::Database;
use crate::domain::{
    catalog::{name_key, LocalizedName},
    ingredient_library::*,
    recipes::{search_name, valid_id},
    AppError,
};
use rusqlite::{params, Connection, Row};

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
 )";
const FILTER: &str = "(?3='all' OR l.origin=?3) AND (?4 IS NULL OR EXISTS(SELECT 1 FROM members m JOIN tree t ON t.id=m.category_id WHERE m.id=l.id AND m.origin=l.origin AND t.root=?4))";

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
            &format!("{BASE} SELECT count(*) FROM searched l WHERE {FILTER}"),
            params![tr, en, query.origin, query.category_id],
            |r| r.get(0),
        )?;
        let mut items = {
            let mut s=tx.prepare(&format!("{BASE} SELECT l.id,l.origin,l.recipe_id,l.name,l.en,l.notes,l.preferred_unit,l.revision,l.ingredient_type FROM searched l WHERE {FILTER} ORDER BY l.search_name,l.origin,l.id LIMIT ?5 OFFSET ?6"))?;
            let rows = s
                .query_map(
                    params![
                        tr,
                        en,
                        query.origin,
                        query.category_id,
                        query.limit,
                        query.offset
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
            let sql=format!("{BASE}, matches AS (SELECT DISTINCT t.root,l.origin,l.id FROM searched l JOIN members m ON m.id=l.id AND m.origin=l.origin JOIN tree t ON t.id=m.category_id WHERE ?3='all' OR l.origin=?3) SELECT c.id,c.parent_id,c.stable_key,c.name_tr,c.name_en,count(m.id) FROM ingredient_categories c LEFT JOIN matches m ON m.root=c.id GROUP BY c.id ORDER BY (SELECT count(*) FROM tree t WHERE t.id=c.id) DESC,c.name_tr,c.id");
            let mut s = tx.prepare(&sql)?;
            let rows = s
                .query_map(params![tr, en, query.origin], |r| {
                    Ok(CategoryCount {
                        category: category(r, 0)?,
                        count: r.get(5)?,
                    })
                })?
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
        let mut value=tx.query_row(&format!("{BASE} SELECT id,origin,recipe_id,name,en,notes,preferred_unit,revision,ingredient_type FROM library WHERE id=?3 AND origin=?4"),params!["","",id,origin],item).map_err(|e| match e { rusqlite::Error::QueryReturnedNoRows=>AppError::new("NOT_FOUND","errors.notFound",true), _=>e.into() })?;
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
        };
        if let Some(catalog_id) = catalog_id {
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
