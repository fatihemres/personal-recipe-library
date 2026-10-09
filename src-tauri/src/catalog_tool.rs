//! Offline preparation/import entry point. No network client or renderer filesystem command.
use crate::{
    domain::{catalog::*, AppError},
    persistence::{migrations::checksum, Database},
};
use serde::Deserialize;
use std::{collections::BTreeMap, path::Path};
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Registry {
    sources: Vec<Clearance>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Clearance {
    id: String,
    seed_approved: bool,
    license: String,
    pinned_version: Option<String>,
    #[serde(default)]
    pinned_versions: Vec<String>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Evidence {
    external_id: String,
    description: String,
}
fn read(root: &Path, artifact: &Artifact) -> Result<Vec<u8>, AppError> {
    if artifact.file.is_empty()
        || artifact.file.contains(['/', '\\'])
        || artifact.file == ".."
        || artifact.sha256.len() != 64
    {
        return Err(catalog_error("CATALOG_ARTIFACT"));
    }
    let path = root.join(&artifact.file);
    if std::fs::symlink_metadata(&path)?.file_type().is_symlink()
        || path.metadata()?.len() > 32 * 1024 * 1024
    {
        return Err(catalog_error("CATALOG_ARTIFACT"));
    }
    let bytes = std::fs::read(path)?;
    use sha2::{Digest, Sha256};
    if format!("{:x}", Sha256::digest(&bytes)) != artifact.sha256 {
        return Err(catalog_error("CATALOG_CHECKSUM"));
    }
    Ok(bytes)
}
pub(crate) fn load(manifest: &Path) -> Result<VerifiedSeed, AppError> {
    if std::fs::symlink_metadata(manifest)?
        .file_type()
        .is_symlink()
        || manifest.metadata()?.len() > 1024 * 1024
    {
        return Err(catalog_error("CATALOG_ARTIFACT"));
    }
    let raw = std::fs::read_to_string(manifest)?;
    verify_package(&raw, |a| {
        read(
            manifest
                .parent()
                .ok_or_else(|| catalog_error("CATALOG_ARTIFACT"))?,
            a,
        )
    })
}
fn verify_package(
    raw: &str,
    reader: impl Fn(&Artifact) -> Result<Vec<u8>, AppError>,
) -> Result<VerifiedSeed, AppError> {
    let m: SeedManifest =
        serde_json::from_str(raw).map_err(|_| catalog_error("CATALOG_INVALID"))?;
    let registry: Registry = serde_json::from_str(include_str!("../../catalog/sources.json"))
        .map_err(|_| AppError::integrity())?;
    let bytes = reader(&m.artifact)?;
    let data: SeedData =
        serde_json::from_slice(&bytes).map_err(|_| catalog_error("CATALOG_INVALID"))?;
    if m.evidence.len() != m.sources.len() {
        return Err(catalog_error("CATALOG_PROVENANCE"));
    }
    for source in &m.sources {
        if !registry.sources.iter().any(|s| {
            s.id == source.id
                && s.seed_approved
                && s.license == source.license
                && (s.pinned_version.as_deref() == Some(&source.version)
                    || s.pinned_versions.contains(&source.version))
        }) {
            return Err(catalog_error("CATALOG_LICENSE"));
        }
        let artifact = m
            .evidence
            .get(&source.id)
            .ok_or_else(|| catalog_error("CATALOG_PROVENANCE"))?;
        if source.artifact_sha256 != artifact.sha256 {
            return Err(catalog_error("CATALOG_CHECKSUM"));
        }
        let evidence: Vec<Evidence> = serde_json::from_slice(&reader(artifact)?)
            .map_err(|_| catalog_error("CATALOG_PROVENANCE"))?;
        let mut records = BTreeMap::new();
        for e in evidence {
            if records.insert(e.external_id, e.description).is_some() {
                return Err(catalog_error("CATALOG_DUPLICATE"));
            }
        }
        for i in &data.ingredients {
            for p in i.provenance.iter().filter(|p| p.source_id == source.id) {
                if p.source_version != source.version
                    || records.get(&p.external_id) != Some(&p.description)
                {
                    return Err(catalog_error("CATALOG_PROVENANCE"));
                }
            }
        }
    }
    let artifact_hash = m.artifact.sha256.clone();
    let seed = VerifiedSeed {
        manifest: m,
        data,
        manifest_hash: checksum(raw),
        artifact_hash,
    };
    seed.validate()?;
    Ok(seed)
}
/// Imports a checked local package into an explicitly chosen app-data directory.
/// Validation fixtures require explicit opt-in and never run during app startup.
pub fn import(
    directory: &Path,
    manifest: &Path,
    allow_validation: bool,
) -> Result<ImportReport, AppError> {
    let seed = load(manifest)?;
    if seed.manifest.purpose == "validation" && !allow_validation {
        return Err(catalog_error("CATALOG_VALIDATION_ONLY"));
    }
    Database::open(directory)?.import_catalog(&seed)
}

/// Same checksum/provenance/license validation as the CLI, with compile-time offline artifacts.
pub(crate) fn bundled() -> Result<VerifiedSeed, AppError> {
    verify_package(
        include_str!("../../catalog/production/manifest.json"),
        |a| {
            let bytes: &[u8] = match a.file.as_str() {
                "ingredients.json" => include_bytes!("../../catalog/production/ingredients.json"),
                "usda-records.json" => include_bytes!("../../catalog/production/usda-records.json"),
                "curation-records.json" => {
                    include_bytes!("../../catalog/production/curation-records.json")
                }
                _ => return Err(catalog_error("CATALOG_ARTIFACT")),
            };
            use sha2::{Digest, Sha256};
            if format!("{:x}", Sha256::digest(bytes)) != a.sha256 {
                return Err(catalog_error("CATALOG_CHECKSUM"));
            }
            Ok(bytes.to_vec())
        },
    )
}
