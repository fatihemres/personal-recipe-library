# Architecture

Status: M1, M2A and M2B implemented, 2026-10-08; M3 and later domain modules remain proposed. MASTER_SPEC.md governs scope. Preserve the existing Tauri 2/React/TypeScript/Vite starter and npm lockfile. No architectural incompatibility currently justifies replacing it. Validate dependency compatibility at introduction rather than upgrading everything now.

## Boundaries and technology decisions

React presentation → typed IPC client → Rust commands → application services/domain → repositories → SQLite. Rust media/import/backup adapters own filesystem access. The renderer never receives unrestricted SQL or arbitrary filesystem execution. All persistent application records/settings/drafts live in SQLite; media bytes are the explicit filesystem exception with DB references.

- Frontend: `src/app` for shell/providers/navigation, `src/features/{recipes,ingredients,beverages,pantry,shopping,planner,collections,dashboard,settings}`, `src/shared/{ui,i18n,contracts,api}`. Cross-feature orchestration uses services/contracts, not imports of another feature's private components.
- Backend: `src-tauri/src/{commands,domain,services,persistence,media,imports,backup}`; SQL migrations in `src-tauri/migrations`; validated bundled seeds in resources; data-build tooling separate from runtime. Create only modules needed by an actual increment.
- Retain React/TypeScript/Vite; add compatible Tailwind/shadcn components incrementally with theme tokens and accessible primitives. Turkish translation resources from the first shell, English resource structure prepared. No remote fonts/assets needed for core operation.
- Zod validates frontend DTOs/forms; Rust independently validates every command, importer, quantity, and constraint. Serde DTOs and contract tests prevent drift; decimal quantities serialized as strings. Error envelope: stable code, localized message key, field details, recoverability; no private paths/data leaked into UI/logs.
- Recommend `rusqlite` with bundled SQLite and confirmed FTS5 support, including backup APIs, behind Rust repositories. One managed DB worker serializes writes and moves blocking operations off the UI thread. This keeps transaction, decimal, migration, and backup policy in one owner. Avoid exposing a generic SQL plugin to UI. Final crate version/features require a compatibility/build check in M1.
- Rust exact rational unit factors plus a decimal library for quantities/density/calculations. Rust is authoritative; preview UI calls shared commands, not a divergent conversion implementation.
- Vitest for UI/domain-facing tests; Rust tests with real temporary SQLite/files for persistence. Playwright for applicable renderer interaction tests. Native automation must exercise real commands/storage separately; do not label mocked browser IPC as native E2E.

## Proposed logical schema

This is an entity design, not executable DDL. Detailed columns/types/indexes/migration SQL and data dictionary are M1 deliverables, expanded with each owning phase. Stable text UUIDs for personal entities; deterministic namespace/source-based seed IDs. UTC instants for audit events, local calendar dates for meals/expiry, explicit units for durations and temperatures. Locale never changes identity.

| Entity group | Relationships and principal fields |
| --- | --- |
| `recipes` | kind food/beverage, title, yield, notes format/content, source/attribution, favorite, archive/delete timestamps, created/updated timestamps, revision counter |
| `recipe_versions`, `recipe_relations` | immutable schema-versioned aggregate snapshots per recipe; variation-of/related links between independent editable recipes; restore creates a new revision, not a rewrite of history |
| `recipe_ingredients`, `recipe_steps`, `step_ingredients` | ordered ingredient lines and steps; ingredient/product ID, quantity/display unit/preparation note; step duration/temp/equipment/technique/tips; join references a line in the same recipe |
| `ingredients`, `ingredient_products` | canonical identity separate from branded product; type, preferred unit, metadata; products reference ingredient with brand/barcode/ABV overrides |
| `ingredient_names`, `ingredient_aliases`, `ingredient_categories`, `ingredient_category_memberships` | localized names/aliases, normalized search keys; category hierarchy with cycle prevention; ingredient many-to-many category links |
| `ingredient_dimensions`, `ingredient_conversions`, `ingredient_nutrients`, `ingredient_allergens`, `ingredient_dietary_metadata` | permitted dimensions; density/context/product/source/validity; nutrient value/unit/basis/state; allergen present/absent/unknown and evidence; dietary provenance |
| `categories`, `recipe_categories`, `tags`, `recipe_tags`, `facets`, `recipe_facets` | hierarchical food/beverage taxonomy; many-to-many recipe categories/tags; distinct cuisine/country/region/method/diet/occasion/difficulty vocabularies |
| `beverage_details`, beverage detail joins | one-to-one recipe extension: classification, servings, format/batch volume, temperature/glassware/ice/garnish/origin/flavor/notes; spirits/methods; dilution input and ABV calculation assumptions/status |
| `units`, `measurement_presets`, `measurement_profiles` | dimension, exact factor/affine rule, localized labels; personal household/bar quantities and versioned definitions |
| `media`, `recipe_media`, `step_media` | UUID, relative storage key, checksum, MIME/size, thumbnail key; explicit associations and cover selection; optional video-reference kind |
| `collections`, `collection_members`, `ratings`, `preparation_history`, `saved_filters`, `recipe_private_notes` | many-to-many membership; personal rating; preparation date/portions/note and recipe version link; versioned filter expression; to-try status |
| `pantry_items`, `inventory_movements` | distinct lots linked to ingredient/product, quantity/unit/location/expiry/price/currency; explicit audited stock changes |
| `shopping_lists`, `shopping_items`, `shopping_item_sources` | completion/manual state, computed and display quantities, recipe/planner contribution and pantry-deduction snapshot |
| `meal_plans`, `meal_plan_entries` | calendar date/meal, recipe/portions, ordering and shopping-list linkage |
| `preferences`, `editor_drafts` | locale/theme/display preferences; versioned unfinished editor state, original recipe revision, recovery timestamp |
| `external_sources`, `source_records`, `seed_versions`, `import_runs`, `import_conflicts`, `ingredient_overrides` | source/license/version/checksum; upstream identity mappings; atomic import status/counts/errors; explicit conflicts; user overrides separate from incoming source fields |
| `schema_migrations`, FTS tables | migration version/checksum/applied time; rebuildable recipe and ingredient search documents |

Use foreign keys on every connection, NOT NULL/CHECK constraints for required fields, ABV range 0–100, positive densities/servings, nonnegative stock and valid dimensions; optional quantity may be unknown rather than guessed. Unique membership pairs, alias identity scoped by ingredient/locale, unit codes and source-record identities; names/barcodes alone do not uniquely identify all canonical ingredients. Index FK columns, recipe status/update/kind, taxonomy parents, stock ingredient/expiry, meal dates, source IDs. Stable line ordering is unique per recipe and changed transactionally. Cross-recipe step links and hierarchy cycles require service-level validation plus focused integrity tests.

CASCADE only for exclusively owned rows/join links at deliberate permanent purge. RESTRICT deletion of referenced ingredient/unit/source definitions; offer archive or explicit replacement. Soft deletion preserves history and media; archive is distinct from trash. History/preparation records retain version meaning. Merge ingredients explicitly remaps references transactionally; never auto-merge by fuzzy similarity.

## Quantities, scaling, ABV

Store decimal quantity text and canonical dimension/unit separately from original display quantity/unit and preset-definition snapshot. Avoid SQLite REAL for authoritative quantities; SQL statistics need deliberate decimal aggregation in Rust. Base dimensions mL, g, count, Celsius; exact rational factors for standardized units. Fahrenheit is affine, not a multiplicative quantity. Treat pinch/drop/dash/shot/cup/spoons/packets as count or explicit profile/context-defined measures; there is no universal household or bar volume. Ingredient-specific piece weight or package size is not density.

Cross mass/volume only with positive applicable density including preparation state/source; otherwise return an explanation. Scale original quantities by target/base yield, retaining unrounded values. Display sensible fractions and practical whole-count suggestions without silently changing mathematical totals; users confirm adjustments. Turkish decimal-comma parsing must reject ambiguity instead of guessing.

Estimated ABV = ethanol volume / final beverage volume × 100, with ethanol derived from known liquid volume and product/ingredient ABV. Known nonalcoholic liquid contributes zero only with evidence, not absent metadata. Known ice melt/dilution contributes volume, solid garnish does not automatically do so. Unknown volumes/ABVs/dilution return unavailable or an explicitly assumption-based estimate; retain inputs and assumptions. Heating/evaporation, mixing contraction, and uncertain melt preclude exact claims. Product overrides do not mutate canonical ABV.

## Search and localization

Store original Unicode names plus NFC normalized keys. Implement shared Turkish-aware search normalization for İ/i and I/ı; accent-tolerant aliases are additional keys, not identity rewrites. Use an explicit Turkish sort strategy verified across native and JS paths; SQLite built-in NOCASE is not the solution. FTS5 indexes recipe titles, notes, steps, ingredient/localized aliases; rebuild/update within writes and exclude trashed records by policy. Prefix FTS plus bounded candidate trigram/edit-distance ranking supports offline typo tolerance. Escape user query into permitted FTS syntax, bind parameters, paginate and cap work. Do not treat FTS5 as automatically typo tolerant or Turkish-aware. [SQLite FTS5 documentation](https://www.sqlite.org/fts5.html).

## Catalog ingestion and licensing

Build-time source download → pinned raw artifact/checksum/license review → normalized staging → validation/identity mappings → Turkish terminology review/deduplication → packaged versioned seed → transactional runtime import. Install contains seed/resources, never depends on API access. Preserve user field overrides across seed upgrades; source deletions do not delete personal references. Report inserted/updated/unchanged/rejected/conflicted and unique validated totals per category, with overlapping membership counts distinguished from overall unique totals.

| Source | Recommended use and unresolved gate |
| --- | --- |
| USDA FoodData Central | Candidate food/nutrient records. Data types have different purposes and evidence; prefer curated generic-food records, not merging every branded food into canonical ingredients. Verify terms on the exact downloaded artifact before redistribution; no license clearance is claimed here. [USDA data documentation](https://fdc.nal.usda.gov/data-documentation/) |
| FoodOn | Candidate taxonomy/aliases and cross-references; ontology classes are not automatically usable culinary ingredients. Repository declares CC BY 4.0; preserve attribution/change notices and review imported ontology dependencies per pinned release. [FoodOn repository](https://github.com/FoodOntology/foodon) |
| Open Food Facts | Optional branded-product enrichment candidate, not primary canonical catalog. Database ODbL, contents DbCL, images CC BY-SA; assess derivative-database/share-alike and attribution obligations before combining or distributing. A separate source layer alone does not prove compliance. [Official licensing guide](https://openfoodfacts.github.io/documentation/docs/Product-Opener/api/tutorials/license-be-on-the-legal-side/) |
| Beverage/Turkish references | No bulk source approved yet. Review explicit redistribution rights; use independently curated factual records with documented provenance where lawful. Do not scrape protected recipe prose/images or invent ABV/nutrient data. |

License review is a release gate, not permission to omit the extensive V1 catalog. Set a concrete coverage target/checklist before M3 ingestion; if a source fails, find a lawful alternative and record the decision. No third-party data was imported during this task.

## Migrations, backups, files, and recovery

Transactional ordered migrations with checksums and an explicit supported-schema range. Snapshot existing DB before schema changes, check integrity/foreign keys after upgrade, test every supported prior version. Refuse newer schemas without mutation. SQLite table-rebuild migrations must preserve indexes/triggers/data. See [SQLite ALTER TABLE](https://www.sqlite.org/lang_altertable.html). Schema migrations and seed upgrades have separate versions. No assumed automatic down-migration; restore an older backup with the matching app, documenting data-loss boundaries.

Use SQLite online backup API to create a consistent snapshot, not a live DB-file copy in WAL mode. Quiesce writes/media mutations across snapshot and media collection; archive manifest includes format/app/schema/seed versions, relative paths, lengths and hashes. See [SQLite backup API](https://www.sqlite.org/backup.html). Stage restore in a separate directory; validate archive paths, symlinks, size limits, checksums, DB integrity/FKs and schema; show preview/conflicts, obtain replacement confirmation, retain recovery snapshot, close connections, swap data generation with a crash-recovery journal and reopen. Cross-platform rename assumptions must be tested; never delete old data before successful reopen.

Media imports validate type/size and generate thumbnails off-thread, strip sensitive metadata as appropriate, use UUID relative names (Unicode display filenames separately). Temporary files + atomic finalize/journal compensate for SQL/filesystem transaction gaps. Garbage collect only unreferenced media after trash/history/drafts/backups policy allows it. Rust validates canonical paths/scoped roots and permissions/ACLs; archive traversal and Windows reserved/case-colliding filenames are tested. External URL opening is explicit; no automatic fetching.

## Platform and test strategy

Bundle SQLite/FTS5 consistently, use Tauri app-data paths, and keep WebView-specific checks on both hosts. Desktop macOS may use Command Line Tools; Windows development needs C++ Build Tools and WebView2. Verify an offline WebView2 installation/distribution strategy for clean Windows machines so installation does not undermine offline functionality. [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/).

Playwright renderer tests cannot alone prove Tauri/WKWebView/native IPC behavior. Add a native-driver feasibility check in M1; current Tauri docs describe WebdriverIO embedded service across platforms and native tauri-driver Windows/Linux support. Select and validate a real-IPC native route or document manual native evidence until automation works; no completion claim based on mocks. [Tauri WebDriver](https://v2.tauri.app/develop/tests/webdriver/).

Build/sign macOS Apple Silicon and supported Intel, Windows on native CI hosts. Confirm runner/SDK/OS support matrix rather than promising unsupported Intel versions. macOS public distribution needs signing/notarization with appropriate credentials; Windows signing needs an explicit certificate/service strategy. Secrets stay in CI secret storage, unsigned test builds labeled. [macOS signing](https://v2.tauri.app/distribute/sign/macos/), [Windows signing](https://tauri.app/distribute/sign/windows/).

## Risk register

| Risk | Mitigation and closure gate |
| --- | --- |
| Catalog coverage, Turkish terminology, product/canonical confusion | Reviewed coverage matrix and real count report; source mappings and manual ambiguous-merge review; M3 gate |
| License incompatibility or missing beverage rights | Exact artifact license notices and redistribution review; lawful substitute; release blocked until cleared |
| Household units/density precision and fractional counts | Exact factors, explicit contexts, null unknowns, preset snapshots, property/golden tests; M2/M6 gates |
| ABV uncertainty | Evidence/status/assumptions, product overrides, unavailable results; M6 tests |
| Turkish FTS/sorting inconsistency | Shared normalization policy, targeted İ/I/ı/i and composed/decomposed tests on both OSes; M4 gate |
| Migrations/data loss and seed overwrites | Backup/checksum/transaction gates, overrides, old-version fixtures, failure injection; every persistence milestone |
| SQL/filesystem atomicity and WAL backup | Staging journals, online backup, consistent media snapshot, interrupted restore tests; M5/M9 |
| Native testing/signing/Intel runners/offline WebView2 | Early M1 feasibility/build checks, native release matrix and credential gates; M11 |
| Large scope/performance | Small vertical slices, paginated queries/thumbnail loading, representative offline datasets and recorded budgets; no scope reduction |
| Dependency compatibility | Pin versions when added; retain working lockfiles; test Tailwind/shadcn/TS/Vite/Rust integration before commitment |

## M1 implementation decisions and evidence

- `rusqlite` 0.40.2 with bundled SQLite/backup, SHA-256 via sha2 0.10.9, Tokio oneshot responses and a dedicated Rust worker implement the recommended ownership boundary. SQLite 3.53.2/FTS5 verified on local arm64 macOS. No SQL plugin or generic filesystem command is exposed.
- Bootstrap schema is only migration metadata/preferences, with exact SQL checksums, `user_version`, read-only existing-data validation, IMMEDIATE transactional migrations, online pre-upgrade snapshots, foreign keys and WAL. See DATABASE_SCHEMA.md for behavior. Full backup/restore and recipe tables remain future milestones.
- Frontend adds Zod 4.6.5, Tailwind/Vite plugin 4.3.3, Lucide 1.53.0, and a locally owned shadcn-style Radix Slot/CVA Button with clsx/tailwind-merge. No full component framework or template regeneration. Existing React/Vite/TypeScript versions retained; compatible installed versions are locked in package-lock.json.
- Tooling: Vitest 5.0.3/jsdom/Testing Library; ESLint 10 with typescript-eslint/hooks; Playwright 1.64.0 browser-only coverage. Node typings cover tooling configs. Tauri's test feature is dev-only for real command-dispatch/SQLite tests with a MockRuntime window.
- The original greeting verification is replaced by real bootstrap diagnostics and preference persistence. Unused opener runtime/dependency/capability removed. CSP now scopes production assets/IPC; native window minimum size and Turkish native menu resources added.
- Native UI verification uses the unsigned packaged debug .app with no Vite server: live diagnostics and light/dark switching, full quit/relaunch with saved dark theme. Browser mock/client tests are explicitly separate from this evidence. Windows/Intel/signing/installer/native driver automation remain unverified.

## M2A implementation decisions — 2026-10-08 (historical; M2B extends these below)

The existing Tauri/React/Rust worker architecture is retained. Typed command closures are queued to the same database-owning thread, using oneshot results; no renderer SQL/filesystem permissions were added. Domain validation lives in `domain/recipes.rs`, parameterized transactional repositories in `persistence/recipes.rs`, and adapters in `commands.rs`. Frontend contracts/API clients remain separate from library/editor/ingredient controls and the reusable native HTML confirmation dialog. Settings navigation keeps the recipe UI mounted to preserve in-memory edits.

New Rust dependencies are pinned uuid 1.27.0 (`v4`, stable identities) and unicode-normalization 0.1.25 (NFC ingredient matching); uuid was already present transitively. No frontend runtime dependencies were added. A temporary pinned formatter was used without modifying npm dependencies. SQLite 3.53.2 remains bundled. See DATABASE_SCHEMA.md and DATA_DICTIONARY.md for the actual schema/IPC.

Measurements are authoritative positive decimal strings, not floating-point. M2A deliberately preserves entered quantities and selected units without conversion/scaling; units carry exact standard factor metadata for M6. Canonical/normalized quantity storage is not yet implemented and is reserved for an additive measurement migration. No mass/volume conversion is inferred. Personal ingredient definitions survive recipe deletion/cancel and will be extended with separate source/catalog/localization entities at M3.

Optimistic revision checks prevent silent stale writes. Owned child rows are replaced atomically with stable IDs and explicit order. Soft delete and restore retain aggregate data; no permanent purge command exists. Personal ingredient creation reuses obvious normalized duplicates. Catalog ingestion and advanced fuzzy search remain unimplemented. Current recipe lists are unpaginated, and title filtering is renderer-side; summary/pagination and FTS performance work must precede catalog-scale library claims. Full durable drafts remain M2B, so save before quitting/crashing.

## M2B reliability decisions — 2026-10-08

The existing React → Zod-validated IPC → thin Rust command → dedicated storage worker → repository/SQLite architecture is retained. No new dependencies or capabilities were required. Reliability models/repositories compose the M2A aggregate writer within one transaction; saved decimal quantities and original unit references remain unchanged.

`DraftSession` serializes debounced/periodic writes, explicit Save and Discard for one stable session UUID. `useDurableDraft` checkpoints after 500 ms of idle input and every 2 seconds while editing, retaining partially entered decimal strings and blank steps. Draft revision CAS prevents two recovered editors from silently replacing each other. The original recipe ID/base revision stays immutable in a session; stale recipe saves fail. An explicitly requested save-as-new generates independent recipe/child IDs; a shared-session conflict forks a new session without deleting another editor's work.

A successful Save atomically writes the recipe aggregate and closes its draft, removing its payload/references. A tiny closed-session tombstone rejects delayed autosaves. This is intentional; deleting that row immediately would permit a queued first-write to recreate an obsolete draft. Tombstone retention/compaction needs a separately designed safe policy. Draft payload format 1 is bounded JSON with separate FK-backed ingredient references, not production recipe rows. Schema upgrades must migrate payload formats explicitly rather than deserialize incompatible drafts silently.

Native window close and app ExitRequested (including Cmd-Q) request the registered renderer guard, await its latest draft flush, and only then authorize Rust exit. Failure keeps the editor open with Turkish feedback. Force-quit/renderer failure cannot flush unacknowledged keystrokes; acknowledged SQLite snapshots recover after unexpected process termination. Debounce is a bounded recovery window, not per-keystroke durability or a portable backup.

Archive and deletion are independent timestamps with revision increments. Trash hides both active and archived records; restoration retains the previous archive state. Purge is restricted to deleted recipes at the expected revision and cascades only owned steps/lines. Edit drafts survive with NULL recipe FK after purge, preserving original ID/base in their payload for explicit recovery-as-new. Future media/history entities must extend purge safety before those features ship.

Personal ingredient edits use their own optimistic revision, notes and preferred-unit FK. Referenced definitions (saved recipes, including archive/trash, and active drafts) cannot be deleted. Renaming changes display names everywhere without changing IDs or measurements. A normalized duplicate rename is a localized conflict, never an automatic merge. Personal records stay independent of future catalog provenance/override entities.

Search normalizes both stored names and queries with NFC, Turkish-aware case mapping and whitespace handling, using a parameterized substring query. SQLite NOCASE is insufficient for Turkish. No M2A matching defect reproduced against saved Şeker; M2B adds real catalog counts, distinct empty/no-match/error states, keyboard autocomplete, exact-name priority and an explicit 50-result refinement notice. Typo tolerance, aliases and catalog indexing remain M3/M4. Lists remain unpaginated; performance work is still required before large-library claims.
