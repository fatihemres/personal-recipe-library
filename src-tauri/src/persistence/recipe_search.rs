use super::Database;
use crate::domain::{
    catalog::name_key,
    recipe_search::{RecipeSearchItem, RecipeSearchPage, RecipeSearchQuery},
    recipes::search_name,
    AppError,
};
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};

fn sort_key(title: &str) -> String {
    const LETTERS: &str = "abcçdefgğhıijklmnoöprsştuüvyz";
    let normalized = search_name(title);
    normalized
        .chars()
        .map(|c| {
            if c == ' ' {
                "000000".to_string()
            } else if let Some(index) = LETTERS.chars().position(|letter| letter == c) {
                format!("{:06x}", index + 1)
            } else if c.is_ascii_digit() {
                format!("{:06x}", 40 + c.to_digit(10).unwrap_or(0))
            } else {
                format!("{:06x}", 0x100 + u32::from(c))
            }
        })
        .collect()
}

pub(super) fn index_recipe(conn: &Connection, id: &str) -> Result<(), AppError> {
    let header: Option<(String, Option<String>, Option<String>)> = conn
        .query_row(
            "SELECT title,description,notes FROM recipes WHERE id=?1",
            [id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()?;
    let Some((title, description, notes)) = header else {
        conn.execute("DELETE FROM recipe_search_docs WHERE recipe_id=?1", [id])?;
        return Ok(());
    };
    let mut body = vec![description.unwrap_or_default(), notes.unwrap_or_default()];
    let mut steps =
        conn.prepare("SELECT instructions FROM recipe_steps WHERE recipe_id=?1 ORDER BY position")?;
    for value in steps.query_map([id], |row| row.get::<_, String>(0))? {
        body.push(value?);
    }
    let body = body.join(" ");
    let mut tr = Vec::new();
    let mut en = Vec::new();
    let mut ingredients = conn.prepare(
        "SELECT i.name, tn.name, en.name FROM recipe_ingredients ri
         JOIN ingredients i ON i.id=ri.ingredient_id
         LEFT JOIN catalog_names tn ON tn.ingredient_id=i.catalog_id AND tn.locale='tr'
         LEFT JOIN catalog_names en ON en.ingredient_id=i.catalog_id AND en.locale='en'
         WHERE ri.recipe_id=?1 ORDER BY ri.position",
    )?;
    for row in ingredients.query_map([id], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, Option<String>>(1)?,
            row.get::<_, Option<String>>(2)?,
        ))
    })? {
        let (personal, turkish, english) = row?;
        tr.push(personal.clone());
        en.push(personal);
        if let Some(value) = turkish {
            tr.push(value);
        }
        if let Some(value) = english {
            en.push(value);
        }
    }
    let mut aliases = conn.prepare(
        "SELECT a.locale,a.name FROM recipe_ingredients ri JOIN ingredients i ON i.id=ri.ingredient_id
         JOIN catalog_aliases a ON a.ingredient_id=i.catalog_id WHERE ri.recipe_id=?1",
    )?;
    for row in aliases.query_map([id], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })? {
        let (locale, value) = row?;
        if locale == "tr" {
            tr.push(value);
        } else {
            en.push(value);
        }
    }
    let mut personal_aliases = conn.prepare(
        "SELECT a.alias FROM recipe_ingredients ri JOIN ingredients i ON i.id=ri.ingredient_id
         JOIN ingredient_personal_aliases a ON a.catalog_id=i.catalog_id WHERE ri.recipe_id=?1",
    )?;
    for value in personal_aliases.query_map([id], |row| row.get::<_, String>(0))? {
        let value = value?;
        tr.push(value.clone());
        en.push(value);
    }
    conn.execute(
        "INSERT INTO recipe_search_docs(recipe_id,title_tr,title_en,body_tr,body_en,ingredients_tr,ingredients_en,sort_key)
         VALUES(?1,?2,?3,?4,?5,?6,?7,?8)
         ON CONFLICT(recipe_id) DO UPDATE SET title_tr=excluded.title_tr,title_en=excluded.title_en,
         body_tr=excluded.body_tr,body_en=excluded.body_en,ingredients_tr=excluded.ingredients_tr,
         ingredients_en=excluded.ingredients_en,sort_key=excluded.sort_key",
        params![id,search_name(&title),name_key("en",&title),search_name(&body),name_key("en",&body),search_name(&tr.join(" ")),name_key("en",&en.join(" ")),sort_key(&title)],
    )?;
    Ok(())
}

pub(super) fn index_recipe_if_available(conn: &Connection, id: &str) -> Result<(), AppError> {
    let available: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='recipe_search_docs')",
        [],
        |row| row.get(0),
    )?;
    if available {
        index_recipe(conn, id)?;
    }
    Ok(())
}

pub(super) fn index_ingredient_recipes(
    conn: &Connection,
    ingredient_id: &str,
) -> Result<(), AppError> {
    let mut stmt =
        conn.prepare("SELECT DISTINCT recipe_id FROM recipe_ingredients WHERE ingredient_id=?1")?;
    let ids = stmt
        .query_map([ingredient_id], |row| row.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    for id in ids {
        index_recipe(conn, &id)?;
    }
    Ok(())
}

impl Database {
    pub(super) fn ensure_recipe_search(&mut self) -> Result<(), AppError> {
        let ready: i64 = self.conn.query_row(
            "SELECT ready FROM recipe_search_state WHERE id=1",
            [],
            |row| row.get(0),
        )?;
        if ready == 0 {
            self.rebuild_recipe_search()?;
        }
        Ok(())
    }
    pub fn rebuild_recipe_search(&mut self) -> Result<(), AppError> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        tx.execute("DELETE FROM recipe_search_docs", [])?;
        let mut stmt = tx.prepare("SELECT id FROM recipes ORDER BY id")?;
        let ids = stmt
            .query_map([], |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        drop(stmt);
        for id in ids {
            index_recipe(&tx, &id)?;
        }
        tx.execute("UPDATE recipe_search_state SET ready=1 WHERE id=1", [])?;
        tx.commit()?;
        Ok(())
    }
    pub fn search_recipes(&self, query: RecipeSearchQuery) -> Result<RecipeSearchPage, AppError> {
        query.validate()?;
        let tr = search_name(&query.text);
        let en = name_key("en", &query.text);
        let fts = fts_expression(&tr, &en);
        let text = query.text.trim();
        let where_clause = "FROM recipes r JOIN recipe_search_docs d ON d.recipe_id=r.id WHERE
          (?1='' OR d.rowid IN (SELECT rowid FROM recipe_fts WHERE recipe_fts MATCH ?2))
          AND ((?3='active' AND r.deleted_at IS NULL AND r.archived_at IS NULL)
            OR (?3='archived' AND r.deleted_at IS NULL AND r.archived_at IS NOT NULL)
            OR (?3='trash' AND r.deleted_at IS NOT NULL))
          AND (?4 IS NULL OR r.kind=?4)
          AND (?5 IS NULL OR EXISTS(SELECT 1 FROM recipe_ingredients ri WHERE ri.recipe_id=r.id AND ri.ingredient_id=?5))
          AND (?6 IS NULL OR substr(r.created_at,1,10)>=?6)
          AND (?7 IS NULL OR substr(r.created_at,1,10)<=?7)
          AND (?8 IS NULL OR substr(r.updated_at,1,10)>=?8)
          AND (?9 IS NULL OR substr(r.updated_at,1,10)<=?9)";
        let params = params![
            text,
            fts,
            query.scope,
            query.kind,
            query.ingredient_id,
            query.created_from,
            query.created_to,
            query.modified_from,
            query.modified_to
        ];
        let tx = self.conn.unchecked_transaction()?;
        let total: i64 =
            tx.query_row(&format!("SELECT count(*) {where_clause}"), params, |row| {
                row.get(0)
            })?;
        let order = match query.sort.as_str() {
            "created" => "r.created_at DESC,r.id",
            "title_asc" => "d.sort_key ASC,r.id",
            "title_desc" => "d.sort_key DESC,r.id",
            "relevance" if !text.is_empty() => "(SELECT bm25(recipe_fts,8.0,8.0,1.0,1.0,3.0,3.0) FROM recipe_fts WHERE rowid=d.rowid AND recipe_fts MATCH ?2) ASC,r.updated_at DESC,r.id",
            _ => "r.updated_at DESC,r.id",
        };
        let sql = format!("SELECT r.id,r.title,r.description,r.kind,r.servings,r.created_at,r.updated_at {where_clause} ORDER BY {order} LIMIT ?10 OFFSET ?11");
        let mut stmt = tx.prepare(&sql)?;
        let items = stmt
            .query_map(
                params![
                    text,
                    fts,
                    query.scope,
                    query.kind,
                    query.ingredient_id,
                    query.created_from,
                    query.created_to,
                    query.modified_from,
                    query.modified_to,
                    query.limit,
                    query.offset
                ],
                |row| {
                    Ok(RecipeSearchItem {
                        id: row.get(0)?,
                        title: row.get(1)?,
                        description: row.get(2)?,
                        kind: row.get(3)?,
                        servings: row.get(4)?,
                        created_at: row.get(5)?,
                        updated_at: row.get(6)?,
                    })
                },
            )?
            .collect::<Result<Vec<_>, _>>()?;
        drop(stmt);
        let suggestions = if total == 0
            && query.ingredient_id.is_none()
            && query.created_from.is_none()
            && query.created_to.is_none()
            && query.modified_from.is_none()
            && query.modified_to.is_none()
            && tr.chars().count() >= 4
            && tr.chars().count() <= 32
        {
            suggest_titles(&tx, &tr, &query.scope, query.kind.as_deref())?
        } else {
            Vec::new()
        };
        Ok(RecipeSearchPage {
            has_more: query.offset + (items.len() as i64) < total,
            items,
            total,
            suggestions,
        })
    }
}

fn fts_expression(tr: &str, en: &str) -> String {
    fn term(value: &str) -> String {
        let words: Vec<String> = value
            .split(|c: char| !c.is_alphanumeric())
            .filter(|word| !word.is_empty())
            .take(8)
            .map(|word| format!("\"{word}\"*"))
            .collect();
        if words.is_empty() {
            "\"m4a_no_match_token_719f\"".into()
        } else {
            words.join(" AND ")
        }
    }
    if tr == en {
        term(tr)
    } else {
        format!("({}) OR ({})", term(tr), term(en))
    }
}

fn suggest_titles(
    conn: &Connection,
    query: &str,
    scope: &str,
    kind: Option<&str>,
) -> Result<Vec<String>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT r.title,d.title_tr FROM recipes r JOIN recipe_search_docs d ON d.recipe_id=r.id
        WHERE ((?1='active' AND r.deleted_at IS NULL AND r.archived_at IS NULL)
          OR (?1='archived' AND r.deleted_at IS NULL AND r.archived_at IS NOT NULL)
          OR (?1='trash' AND r.deleted_at IS NOT NULL)) AND (?2 IS NULL OR r.kind=?2)
          AND substr(d.title_tr,1,1)=substr(?3,1,1) ORDER BY r.updated_at DESC,r.id LIMIT 200",
    )?;
    let mut scored = stmt
        .query_map(params![scope, kind, query], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .filter_map(|(title, key)| {
            let max = if query.chars().count() > 6 { 2 } else { 1 };
            let distance = edit_distance(query, &key, max)?;
            Some((distance, title))
        })
        .collect::<Vec<_>>();
    scored.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
    scored.dedup_by(|a, b| a.1 == b.1);
    Ok(scored.into_iter().take(3).map(|(_, title)| title).collect())
}

fn edit_distance(a: &str, b: &str, max: usize) -> Option<usize> {
    let left: Vec<char> = a.chars().collect();
    let right: Vec<char> = b.chars().collect();
    if left.len().abs_diff(right.len()) > max || right.len() > 200 {
        return None;
    }
    let mut previous: Vec<usize> = (0..=right.len()).collect();
    for (i, ca) in left.iter().enumerate() {
        let mut current = vec![i + 1; right.len() + 1];
        for (j, cb) in right.iter().enumerate() {
            current[j + 1] = (previous[j + 1] + 1)
                .min(current[j] + 1)
                .min(previous[j] + usize::from(ca != cb));
        }
        if current.iter().copied().min().unwrap_or(max + 1) > max {
            return None;
        }
        previous = current;
    }
    (previous[right.len()] <= max).then_some(previous[right.len()])
}
