use super::{migrations::checksum, Database};
use crate::domain::{
    catalog::*,
    recipes::{conflict, search_name, valid_id},
    AppError,
};
use rusqlite::{params, OptionalExtension, Transaction, TransactionBehavior};
use std::collections::BTreeMap;

impl Database {
    /// Startup never downgrades newer installed data and does not add redundant audit runs.
    pub(crate) fn install_bundled_catalog(&mut self) -> Result<(), AppError> {
        let seed = crate::catalog_tool::bundled()?;
        let latest: i64 = self.conn.query_row(
            "SELECT coalesce(max(version),0) FROM catalog_releases WHERE dataset=?1",
            [&seed.manifest.dataset],
            |r| r.get(0),
        )?;
        if latest > seed.manifest.version {
            return Ok(());
        }
        if latest == seed.manifest.version {
            let hash: String = self.conn.query_row(
                "SELECT manifest_sha256 FROM catalog_releases WHERE dataset=?1 AND version=?2",
                params![seed.manifest.dataset, seed.manifest.version],
                |r| r.get(0),
            )?;
            if hash != seed.manifest_hash {
                return Err(catalog_error("CATALOG_VERSION_CHANGED"));
            }
            return Ok(());
        }
        self.import_catalog(&seed)?;
        Ok(())
    }
    pub fn import_catalog(&mut self, seed: &VerifiedSeed) -> Result<ImportReport, AppError> {
        seed.validate()?;
        let run = uuid::Uuid::new_v4().to_string();
        self.conn.execute(
            "INSERT INTO catalog_import_runs(id,dataset,version,status) VALUES(?1,?2,?3,'running')",
            params![run, seed.manifest.dataset, seed.manifest.version],
        )?;
        let result = self.apply_catalog(seed, &run);
        if let Err(ref error) = result {
            self.conn.execute("UPDATE catalog_import_runs SET status='failed',error_code=?2,finished_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1",params![run,error.code])?;
        }
        result
    }
    fn apply_catalog(&mut self, seed: &VerifiedSeed, run: &str) -> Result<ImportReport, AppError> {
        let m = &seed.manifest;
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let latest: i64 = tx.query_row(
            "SELECT coalesce(max(version),0) FROM catalog_releases WHERE dataset=?1",
            [&m.dataset],
            |r| r.get(0),
        )?;
        if m.version < latest {
            return Err(catalog_error("CATALOG_DOWNGRADE"));
        }
        let previous: Option<String> = tx
            .query_row(
                "SELECT manifest_sha256 FROM catalog_releases WHERE dataset=?1 AND version=?2",
                params![m.dataset, m.version],
                |r| r.get(0),
            )
            .optional()?;
        if let Some(previous) = previous {
            if previous != seed.manifest_hash {
                return Err(catalog_error("CATALOG_VERSION_CHANGED"));
            }
            let report = coverage(&tx, seed, true, 0, 0)?;
            finish_run(&tx, run, &report)?;
            tx.commit()?;
            return Ok(report);
        }
        tx.execute("INSERT INTO catalog_releases(dataset,version,purpose,manifest_sha256,artifact_sha256) VALUES(?1,?2,?3,?4,?5)",params![m.dataset,m.version,m.purpose,seed.manifest_hash,seed.artifact_hash])?;
        for s in &m.sources {
            let old:Option<SourceSnapshot>=tx.query_row("SELECT id,version,name,url,license,attribution,retrieved_at,artifact_sha256 FROM catalog_sources WHERE id=?1 AND version=?2",params![s.id,s.version],|r|Ok(SourceSnapshot{id:r.get(0)?,version:r.get(1)?,name:r.get(2)?,url:r.get(3)?,license:r.get(4)?,attribution:r.get(5)?,retrieved_at:r.get(6)?,artifact_sha256:r.get(7)?})).optional()?;
            if old.as_ref().is_some_and(|old| old != s) {
                return Err(catalog_error("CATALOG_SOURCE_CHANGED"));
            }
            tx.execute("INSERT OR IGNORE INTO catalog_sources(id,version,name,url,license,attribution,retrieved_at,artifact_sha256) VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",params![s.id,s.version,s.name,s.url,s.license,s.attribution,s.retrieved_at,s.artifact_sha256])?;
        }
        let mut remaining: Vec<_> = seed.data.categories.iter().collect();
        while !remaining.is_empty() {
            let before = remaining.len();
            let mut next = vec![];
            for c in remaining {
                let parent = c.parent.as_ref().map(|p| stable_id("category", p));
                if let Some(ref p) = parent {
                    if !tx.query_row(
                        "SELECT EXISTS(SELECT 1 FROM ingredient_categories WHERE id=?1)",
                        [p],
                        |r| r.get::<_, bool>(0),
                    )? {
                        next.push(c);
                        continue;
                    }
                }
                let id = stable_id("category", &c.key);
                let existing: Option<(String, Option<String>)> = tx
                    .query_row(
                        "SELECT origin,dataset FROM ingredient_categories WHERE id=?1",
                        [&id],
                        |r| Ok((r.get(0)?, r.get(1)?)),
                    )
                    .optional()?;
                if existing.is_some_and(|(origin, dataset)| {
                    origin != "catalog" || dataset.as_deref() != Some(&m.dataset)
                }) {
                    return Err(catalog_error("CATALOG_IDENTITY_CONFLICT"));
                }
                tx.execute("INSERT INTO ingredient_categories(id,stable_key,parent_id,name_tr,name_en,origin,dataset) VALUES(?1,?2,?3,?4,?5,'catalog',?6) ON CONFLICT(id) DO UPDATE SET parent_id=excluded.parent_id,name_tr=excluded.name_tr,name_en=excluded.name_en",params![id,c.key,parent,c.tr,c.en,m.dataset])?;
            }
            if next.len() == before {
                return Err(catalog_error("CATALOG_CATEGORY_CYCLE"));
            }
            remaining = next;
        }
        let mut inserted = 0;
        let mut updated = 0;
        for i in &seed.data.ingredients {
            let old:Option<(String,String,String)>=tx.query_row("SELECT canonical_key,dataset,record_sha256 FROM catalog_ingredients WHERE id=?1",[&i.id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).optional()?;
            if old
                .as_ref()
                .is_some_and(|(key, dataset, _)| key != &i.key || dataset != &m.dataset)
            {
                return Err(catalog_error("CATALOG_IDENTITY_CONFLICT"));
            }
            if let Some(ref unit) = i.preferred_unit {
                let dimension: Option<String> = tx
                    .query_row("SELECT dimension FROM units WHERE code=?1", [unit], |r| {
                        r.get(0)
                    })
                    .optional()?;
                if !dimension.is_some_and(|d| i.dimensions.contains(&d)) {
                    return Err(catalog_error("CATALOG_INVALID_UNIT"));
                }
            }
            let hash = checksum(&serde_json::to_string(i).map_err(|_| AppError::integrity())?);
            if old.is_none() {
                inserted += 1;
            } else if old.as_ref().is_some_and(|(_, _, h)| h != &hash) {
                updated += 1;
            }
            tx.execute("INSERT INTO catalog_ingredients(id,canonical_key,canonical_name,ingredient_type,preferred_unit,dataset,catalog_version,record_sha256) VALUES(?1,?2,?3,?4,?5,?6,?7,?8) ON CONFLICT(id) DO UPDATE SET canonical_name=excluded.canonical_name,ingredient_type=excluded.ingredient_type,preferred_unit=excluded.preferred_unit,catalog_version=excluded.catalog_version,record_sha256=excluded.record_sha256",params![i.id,i.key,i.canonical_name,i.ingredient_type,i.preferred_unit,m.dataset,m.version,hash])?;
        }
        for i in &seed.data.ingredients {
            for table in [
                "catalog_names",
                "catalog_aliases",
                "catalog_category_memberships",
                "catalog_dimensions",
                "catalog_relations",
                "catalog_observations",
            ] {
                tx.execute(
                    &format!("DELETE FROM {table} WHERE ingredient_id=?1"),
                    [&i.id],
                )?;
            }
            // Keep old source mappings as historical provenance; never remap an external record silently.
            for p in &i.provenance {
                let owner:Option<String>=tx.query_row("SELECT ingredient_id FROM catalog_provenance WHERE source_id=?1 AND external_id=?2 LIMIT 1",params![p.source_id,p.external_id],|r|r.get(0)).optional()?;
                if owner.is_some_and(|owner| owner != i.id) {
                    return Err(catalog_error("CATALOG_IDENTITY_CONFLICT"));
                }
                tx.execute(
                    "INSERT OR IGNORE INTO catalog_provenance VALUES(?1,?2,?3,?4,?5)",
                    params![
                        i.id,
                        p.source_id,
                        p.source_version,
                        p.external_id,
                        p.description
                    ],
                )?;
            }
            for (locale, name) in &i.names {
                tx.execute(
                    "INSERT INTO catalog_names VALUES(?1,?2,?3,?4)",
                    params![i.id, locale, name, name_key(locale, name)],
                )?;
            }
            for a in &i.aliases {
                tx.execute(
                    "INSERT INTO catalog_aliases VALUES(?1,?2,?3,?4)",
                    params![i.id, a.locale, a.name, name_key(&a.locale, &a.name)],
                )?;
            }
            for c in &i.categories {
                tx.execute(
                    "INSERT INTO catalog_category_memberships VALUES(?1,?2)",
                    params![i.id, stable_id("category", c)],
                )?;
            }
            for d in &i.dimensions {
                tx.execute(
                    "INSERT INTO catalog_dimensions VALUES(?1,?2)",
                    params![i.id, d],
                )?;
            }
            for r in &i.relations {
                tx.execute(
                    "INSERT INTO catalog_relations VALUES(?1,?2,?3)",
                    params![i.id, stable_id("ingredient", &r.key), r.kind],
                )?;
            }
            for o in &i.observations {
                tx.execute(
                    "INSERT INTO catalog_observations VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)",
                    params![
                        i.id,
                        o.kind,
                        o.code,
                        o.value,
                        o.unit,
                        o.basis,
                        o.source_id,
                        o.source_version,
                        o.external_id
                    ],
                )?;
            }
            materialize(&tx, i)?;
        }
        let report = coverage(&tx, seed, false, inserted, updated)?;
        finish_run(&tx, run, &report)?;
        #[cfg(test)]
        if let Ok(marker) = std::env::var("RECIPEATLAS_CATALOG_CRASH_MARKER") {
            std::fs::write(marker, b"uncommitted")?;
            loop {
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        }
        tx.commit()?;
        Ok(report)
    }
    pub fn available_ingredients(
        &self,
        query: &str,
    ) -> Result<crate::domain::reliability::IngredientSearch, AppError> {
        use crate::domain::reliability::{IngredientSearch, PersonalIngredient};
        if query.chars().count() > 200 {
            return Err(catalog_error("CATALOG_INVALID"));
        }
        let tx = self.conn.unchecked_transaction()?;
        let total = tx.query_row("SELECT count(*) FROM ingredients", [], |r| r.get(0))?;
        let mut s=tx.prepare("SELECT i.id,i.name,i.notes,i.preferred_unit,i.revision,i.origin FROM ingredients i WHERE instr(i.search_name,?1)>0 OR EXISTS(SELECT 1 FROM catalog_names n WHERE n.ingredient_id=i.catalog_id AND instr(n.search_key,CASE WHEN n.locale='tr' THEN ?1 ELSE ?2 END)>0) OR EXISTS(SELECT 1 FROM catalog_aliases a WHERE a.ingredient_id=i.catalog_id AND instr(a.search_key,CASE WHEN a.locale='tr' THEN ?1 ELSE ?2 END)>0) ORDER BY (i.origin='personal') DESC,i.search_name,i.id LIMIT 51")?;
        let mut items = s
            .query_map(params![search_name(query), name_key("en", query)], |r| {
                Ok(PersonalIngredient {
                    id: r.get(0)?,
                    name: r.get(1)?,
                    notes: r.get(2)?,
                    preferred_unit: r.get(3)?,
                    revision: r.get(4)?,
                    origin: r.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        let has_more = items.len() > 50;
        items.truncate(50);
        Ok(IngredientSearch {
            items,
            total,
            has_more,
        })
    }
    pub fn catalog_status(&self) -> Result<CatalogStatus, AppError> {
        let tx = self.conn.unchecked_transaction()?;
        let count = |purpose: &str| {
            tx.query_row("SELECT count(*) FROM catalog_ingredients i JOIN catalog_releases r ON r.dataset=i.dataset AND r.version=i.catalog_version WHERE r.purpose=?1",[purpose],|r|r.get::<_,i64>(0))
        };
        let validation = count("validation")?;
        let production = count("production")?;
        Ok(CatalogStatus {
            definitions: validation + production,
            validation_definitions: validation,
            production_definitions: production,
            pending_collisions: tx.query_row(
                "SELECT count(*) FROM catalog_collisions WHERE state='pending'",
                [],
                |r| r.get(0),
            )?,
        })
    }
    pub fn link_personal_catalog(
        &mut self,
        id: &str,
        revision: i64,
        catalog_id: &str,
    ) -> Result<(), AppError> {
        valid_id(id)?;
        valid_id(catalog_id)?;
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let current:Option<String>=tx.query_row("SELECT catalog_id FROM ingredients WHERE id=?1 AND origin='personal' AND revision=?2",params![id,revision],|r|r.get(0)).optional()?.ok_or_else(conflict)?;
        if current.is_some() {
            return Err(conflict());
        }
        // A published catalog row cannot be merged into an existing personal row implicitly.
        if tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM ingredients WHERE catalog_id=?1 AND origin='catalog')",
            [catalog_id],
            |r| r.get::<_, bool>(0),
        )? {
            return Err(catalog_error("CATALOG_LINK_CONFLICT"));
        }
        tx.execute("UPDATE ingredients SET catalog_id=?2,revision=revision+1,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1",params![id,catalog_id])?;
        tx.execute(
            "UPDATE catalog_collisions SET state='linked' WHERE catalog_id=?1 AND personal_id=?2",
            params![catalog_id, id],
        )?;
        tx.commit()?;
        Ok(())
    }
    pub fn customize_catalog_ingredient(
        &mut self,
        input: crate::domain::reliability::IngredientEdit,
    ) -> Result<crate::domain::reliability::PersonalIngredient, AppError> {
        use crate::domain::reliability::PersonalIngredient;
        valid_id(&input.id)?;
        if !text(&input.name, 200)
            || input
                .notes
                .as_ref()
                .is_some_and(|n| n.chars().count() > 2000 || n.contains('\0'))
        {
            return Err(catalog_error("CATALOG_INVALID"));
        }
        let name = input.name.trim();
        let key = search_name(name);
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        if tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM ingredients WHERE id<>?1 AND search_name=?2)",
            params![input.id, key],
            |r| r.get::<_, bool>(0),
        )? {
            return Err(AppError::new(
                "INGREDIENT_DUPLICATE",
                "errors.ingredientDuplicate",
                true,
            ));
        }
        if let Some(ref unit) = input.preferred_unit {
            if !tx.query_row("SELECT EXISTS(SELECT 1 FROM ingredients i JOIN catalog_dimensions d ON d.ingredient_id=i.catalog_id JOIN units u ON u.dimension=d.dimension WHERE i.id=?1 AND u.code=?2)",params![input.id,unit],|r|r.get::<_,bool>(0))? {return Err(catalog_error("CATALOG_INVALID_UNIT"));}
        }
        if tx.execute("UPDATE ingredients SET name=?3,search_name=?4,notes=?5,preferred_unit=?6,revision=revision+1,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1 AND revision=?2 AND origin='catalog'",params![input.id,input.revision,name,key,input.notes,input.preferred_unit])?!=1 {return Err(conflict());}
        for field in ["name", "notes", "preferred_unit"] {
            tx.execute("INSERT INTO ingredient_customizations(ingredient_id,field) VALUES(?1,?2) ON CONFLICT(ingredient_id,field) DO UPDATE SET changed_at=strftime('%Y-%m-%dT%H:%M:%fZ','now')",params![input.id,field])?;
        }
        let result = tx.query_row(
            "SELECT id,name,notes,preferred_unit,revision,origin FROM ingredients WHERE id=?1",
            [&input.id],
            |r| {
                Ok(PersonalIngredient {
                    id: r.get(0)?,
                    name: r.get(1)?,
                    notes: r.get(2)?,
                    preferred_unit: r.get(3)?,
                    revision: r.get(4)?,
                    origin: r.get(5)?,
                })
            },
        )?;
        tx.commit()?;
        Ok(result)
    }
    pub fn create_personal_category(
        &mut self,
        tr: &str,
        en: &str,
        parent: Option<&str>,
    ) -> Result<String, AppError> {
        if !text(tr, 200) || !text(en, 200) {
            return Err(catalog_error("CATALOG_INVALID"));
        }
        if let Some(p) = parent {
            valid_id(p)?;
        }
        let id = uuid::Uuid::new_v4().to_string();
        self.conn.execute("INSERT INTO ingredient_categories(id,stable_key,parent_id,name_tr,name_en,origin) VALUES(?1,?2,?3,?4,?5,'personal')",params![id,format!("personal-{id}"),parent,tr.trim(),en.trim()])?;
        Ok(id)
    }
}
fn materialize(tx: &Transaction<'_>, i: &CatalogEntry) -> Result<(), AppError> {
    let name = &i.names["tr"];
    let mut collisions = std::collections::BTreeSet::new();
    for candidate in i.names.values().chain(i.aliases.iter().map(|a| &a.name)) {
        let mut s=tx.prepare("SELECT id FROM ingredients WHERE origin='personal' AND catalog_id IS NULL AND search_name=?1")?;
        for row in s.query_map([search_name(candidate)], |r| r.get::<_, String>(0))? {
            collisions.insert(row?);
        }
    }
    for id in &collisions {
        tx.execute("INSERT INTO catalog_collisions VALUES(?1,?2,'pending') ON CONFLICT(catalog_id,personal_id) DO NOTHING",params![i.id,id])?;
    }
    let linked: bool = tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM ingredients WHERE catalog_id=?1)",
        [&i.id],
        |r| r.get(0),
    )?;
    // A changed system display name must not introduce an obvious duplicate personal row.
    // Roll back the package for review instead of choosing a merge or changing either identity.
    if !collisions.is_empty() && tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM ingredients WHERE catalog_id=?1 AND origin='catalog' AND name<>?2 AND NOT EXISTS(SELECT 1 FROM ingredient_customizations WHERE ingredient_id=ingredients.id AND field='name'))",
        params![i.id,name],|r|r.get::<_,bool>(0),
    )? {return Err(catalog_error("CATALOG_NAME_CONFLICT"));}
    if !linked && collisions.is_empty() {
        if tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM ingredients WHERE id=?1)",
            [&i.id],
            |r| r.get::<_, bool>(0),
        )? {
            return Err(catalog_error("CATALOG_IDENTITY_CONFLICT"));
        }
        tx.execute("INSERT INTO ingredients(id,name,search_name,origin,preferred_unit,catalog_id,updated_at) VALUES(?1,?2,?3,'catalog',?4,?1,strftime('%Y-%m-%dT%H:%M:%fZ','now'))",params![i.id,name,search_name(name),i.preferred_unit])?;
    }
    tx.execute("UPDATE ingredients SET name=CASE WHEN EXISTS(SELECT 1 FROM ingredient_customizations WHERE ingredient_id=ingredients.id AND field='name') THEN name ELSE ?2 END,search_name=CASE WHEN EXISTS(SELECT 1 FROM ingredient_customizations WHERE ingredient_id=ingredients.id AND field='name') THEN search_name ELSE ?3 END,preferred_unit=CASE WHEN EXISTS(SELECT 1 FROM ingredient_customizations WHERE ingredient_id=ingredients.id AND field='preferred_unit') THEN preferred_unit ELSE ?4 END,revision=revision+1,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE catalog_id=?1 AND origin='catalog' AND ((name<>?2 AND NOT EXISTS(SELECT 1 FROM ingredient_customizations WHERE ingredient_id=ingredients.id AND field='name')) OR (preferred_unit IS NOT ?4 AND NOT EXISTS(SELECT 1 FROM ingredient_customizations WHERE ingredient_id=ingredients.id AND field='preferred_unit')))",params![i.id,name,search_name(name),i.preferred_unit])?;
    Ok(())
}
fn coverage(
    tx: &Transaction<'_>,
    seed: &VerifiedSeed,
    unchanged: bool,
    inserted: usize,
    updated: usize,
) -> Result<ImportReport, AppError> {
    let mut coverage = BTreeMap::new();
    for c in &seed.data.categories {
        let count:i64=tx.query_row("SELECT count(DISTINCT m.ingredient_id) FROM catalog_category_memberships m JOIN catalog_ingredients i ON i.id=m.ingredient_id WHERE m.category_id=?1 AND i.dataset=?2",params![stable_id("category",&c.key),seed.manifest.dataset],|r|r.get(0))?;
        coverage.insert(c.key.clone(), count as usize);
    }
    let available:i64=tx.query_row("SELECT count(DISTINCT catalog_id) FROM ingredients WHERE catalog_id IN (SELECT id FROM catalog_ingredients WHERE dataset=?1)",[&seed.manifest.dataset],|r|r.get(0))?;
    let collisions:i64=tx.query_row("SELECT count(*) FROM catalog_collisions c JOIN catalog_ingredients i ON i.id=c.catalog_id WHERE i.dataset=?1 AND c.state='pending'",[&seed.manifest.dataset],|r|r.get(0))?;
    Ok(ImportReport {
        dataset: seed.manifest.dataset.clone(),
        version: seed.manifest.version,
        purpose: seed.manifest.purpose.clone(),
        unchanged,
        validated: seed.data.ingredients.len(),
        inserted,
        updated,
        available: available as usize,
        collisions: collisions as usize,
        category_coverage: coverage,
    })
}

fn finish_run(tx: &Transaction<'_>, run: &str, report: &ImportReport) -> Result<(), AppError> {
    tx.execute("UPDATE catalog_import_runs SET status='succeeded',report=?2,finished_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?1",params![run,serde_json::to_string(report).map_err(|_|AppError::integrity())?])?;
    Ok(())
}
