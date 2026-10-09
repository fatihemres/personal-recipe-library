# Offline catalog pipeline — M3A

M3A implements a real importer and a small validation package. Broad production population/bundled first-install delivery is M3B; comprehensive discovery/category/collision/customization UI is M3C. Normal application workflows have no network or API-key dependency. No validation records are imported automatically into personal app data.

## Reproduce preparation

Use Python 3 standard library, existing locked Rust dependencies and the official archive:

```sh
curl --fail --location --output /private/tmp/recipeatlas-sr-2018.zip \
  https://fdc.nal.usda.gov/fdc-datasets/FoodData_Central_sr_legacy_food_csv_2018-04.zip
python3 tools/prepare_validation_catalog.py /private/tmp/recipeatlas-sr-2018.zip
```

The generator verifies the full upstream SHA-256, reads only eight selected actual food.csv rows, fails for missing/duplicate IDs, and generates deterministic UTF-8 sorted JSON without timestamps that change per run. Retrieval date is the pinned actual 2026-10-08 retrieval, not the regeneration date. Generated files: `ingredients.json`, `usda-records.json`, `curation-records.json`, `manifest.json`. NOTICE is maintained separately. Manifest pins format 1, dataset, integer release version, purpose validation, filenames, source/version/attribution and SHA-256 of generated/evidence files. Upstream archive URL/hash are in `catalog/sources.json`; the bulk archive is not committed or processed at runtime.

USDA evidence contains exact external IDs/descriptions. Authored curation evidence records label/alias/category/dimension/unit choices. No nutrient files are read. The generated artifacts are committed for offline reproducibility; no network is necessary to import them. Hashes establish integrity, not an authenticated signature or independent proof of translation quality.

## Reproduce import without touching personal data

From repository root:

```sh
cargo run --locked --offline --manifest-path src-tauri/Cargo.toml \
  --example catalog_import -- /private/tmp/recipeatlas-m3a-validation \
  catalog/validation/manifest.json --allow-validation
```

Use a **new explicitly chosen test directory** for clean-install counts. Repeat the exact command to verify idempotency. The CLI outputs real JSON counts; save stdout to a report file when needed. `catalog/validation/coverage.json` is a captured clean import report, not a manually increased target. The example CLI requires the explicit validation flag. No application UI/import permission or network fetch is exposed. Do not aim this validation fixture at personal production app data.

## Trust and validation

1. Reject unsupported JSON shape/format, symlinks, path traversal, oversized files and checksum mismatches. Artifacts are local filenames only; 1 MiB manifest, 32 MiB per artifact; up to 50,000 definitions/2,000 category nodes.
2. Enforce reviewed source allowlist/license and pinned version from compiled registry. Excluded sources cannot be enabled by changing a package's own license claim. Future sources/versions require an explicit repository review/change.
3. Verify every provenance ID/version/description against checksummed evidence. Validate stable IDs, unique keys/localized names, duplicate aliases/memberships/provenance, category/form cycles, exact decimals and compatible preferred units. Known source facts and authored translations are distinguished.
4. Open SQLite using the existing secure initialization/migration path. Prevalidation failures do not create a database or import audit row. Once repository import starts, create a durable `running` audit.
5. Begin IMMEDIATE transaction; reject downgrades or changed content at an existing version. The same manifest hash/version is a no-op with current coverage and its own successful audit.
6. Insert immutable source snapshots and release, parent-first categories, stable canonical identities, names/aliases/relationships/provenance/optional observations. Detect personal name collisions across both languages; never merge or delete personal records. Preserve user overrides and historical provenance.
7. Compute actual direct-category/availability/collision counts; record success and commit all catalog data **together**. SQL/constraint failure rolls back all data and leaves a failed audit with safe structured code.

Unexpected process termination before commit rolls back the data; the initial audit can remain `running` and must be treated as possibly interrupted, not success. A concurrent invocation can also legitimately be running; do not blindly mark every such audit failed. Retry uses the same immutable package; SQLite serialization and version/hash checks prevent partial/double installation. A crash after commit has both complete data and successful audit. Busy timeout uses existing five-second policy; errors never reset the database.

## Updates and coverage

Each changed package needs a greater release version. Source snapshots cannot mutate at the same source/version. A new source artifact needs a new reviewed pinned source version. User-owned records and overrides have priority; no record-removal pass exists. Pre-migration snapshots protect schema changes; the importer itself provides transaction rollback, not a portable backup.

Coverage counts direct memberships, so root `food` has 0 direct members even though its children contain ingredients. A record in both food and beverage categories counts in both; category totals cannot be summed into total ingredients. `validated` counts current package rows; `available` counts resolved recipe-facing canonical identities; pending collision pairs are reported separately. Retained older catalog definitions can remain available across upgrades; distinguish package coverage from cumulative database coverage. `purpose` distinguishes validation from production, including Settings counts.

M3B must extend preparation tools and license allowlist consciously, create a reviewed coverage matrix and actual report, and embed an approved production package for automatic idempotent offline installation through the Rust worker. It must test application packaging, first install and updates while preserving M2B data. This task does not claim production catalog completion.

A catalog upgrade that would change an uncustomized system display name to an existing personal name is rejected with `CATALOG_NAME_CONFLICT`, rolling back the package. Review that collision rather than silently merging, renaming personal data or introducing duplicate display identities. This differs from initial installation, which retains pending candidates without materializing colliding built-in rows.
