# Personal Food & Beverage Library — Master Specification

Status: product baseline, 2026-10-08. Requirements below preserve the supplied specification in normalized Markdown. Architecture recommendations are in ARCHITECTURE.md; delivery and verification are in IMPLEMENTATION_PLAN.md. The baseline was created during the documentation-only planning task. Implementation is authorized one milestone at a time by the current user request; preserve the full product scope while respecting that milestone boundary.

## Scope and version policy

Build a production-quality, comprehensive personal food and beverage management desktop application, developed on macOS and running natively on macOS and Windows from one shared codebase. Responsibility includes architecture, engineering, normalized database, UX/UI, data importers, QA, documentation, packaging, and releases. Deliver functional increments, not throwaway proposals, prototypes, visual mockups, or convincing placeholders.

**All requirements in R01–R20 are V1 unless explicitly identified below.** Optional fields may be empty, and optional modules may be unused; their support is still required. Pantry, shopping lists, meal planning, variations, history, advanced beverage editing, extensive offline catalogs, and portable backups are V1. No feature is silently deferred.

Explicit future support: V1 must externalize strings and support localized ingredient records; a complete English UI translation can be V2. V1 media architecture must accommodate optional video references; a video UI may be V2 and video hosting is not required. Additional proposed V2 enhancements, not part of the requested baseline: optional cloud synchronization/accounts, automatic online catalog refresh, barcode scanning hardware workflows, hosted video, and AI assistance. None may weaken offline operation or replace V1 functionality. Any further scope change requires a recorded user decision.

## R01 — Technology and architecture

Use Tauri 2, React, strict TypeScript, Vite, Tailwind CSS, shadcn/ui, Rust native operations/secure filesystem, SQLite for persistent application data, SQLite FTS5, Zod or equivalent runtime validation, Vitest, applicable Playwright UI/end-to-end tests, Git/GitHub, and GitHub Actions macOS/Windows builds. Use stable compatible dependencies and official documentation; significant incompatibilities must be explained before substituting technology. Modular, maintainable, extensible architecture; database migrations; no large monolithic components or tightly coupled services. Core functionality requires no backend server, account, subscription, or network.

## R02 — Language and localization

Default UI Turkish; future multilingual support, especially English. No interface strings hardcoded into components. Correct Turkish characters in fields, sorting, search, import/export, and filenames. Ingredient identity supports canonical names, localized names, alternatives, synonyms, and search aliases.

## R03 — Desktop UX

Professional, minimal, warm culinary identity; polished spacing, typography, icons, interactions, hierarchy, and long-recipe readability. Functional light/dark themes, keyboard navigation, accessibility, mouse support, responsive desktop window layouts, subtle performant animation. Clear empty/loading/validation/error states. Preserve unfinished edits where appropriate; confirm destructive actions; provide undo/recovery for important operations. Search-first navigation. Quick Add and Advanced Recipe Editor; simple recipes must not require advanced fields. Avoid overwhelming screens, excessive nesting, and complex workflows.

## R04 — Dashboard

Database-backed total recipes, food/beverage counts, favorites, recently added/edited, recently prepared, personal collections, quick actions, useful statistics. Every dashboard item links to a correctly filtered view.

## R05 — Recipe library

Create, view, edit, duplicate, archive, delete food/beverage recipes; soft deletion and trash recovery. Cards/list views, sorting/filtering, advanced full-text search, favorites, personal ratings, rich notes, variations, related recipes, history, ingredient substitutions, portion adjustment, cover/step images, optional source URLs and attribution.

## R06 — Food taxonomy

Hierarchical extensible taxonomy: breakfast, appetizers, soups, salads, main dishes, meat, poultry, seafood, vegetable dishes, legumes, rice, pasta, baked dishes, bread and dough, sauces, side dishes, desserts, snacks, fermented foods, preserves, other user-defined categories. Multiple categories and tags per recipe. Separate cuisine, country, region, preparation method, dietary suitability, occasion, difficulty fields. Never assume one category per recipe.

## R07 — Beverage Studio

Specialized alcoholic/non-alcoholic editor. Categories: cocktails, mocktails, shots, coffee, tea, smoothies, milkshakes, juices, lemonades, syrups, hot drinks, cold drinks, custom categories.

Store alcohol classification; alcohol types/spirits; ABV; serving size/count; individual/shared format; total batch volume; temperature; glassware; ice type/quantity; preparation method; garnish; country/region; traditional/inspired/custom origin; flavor profile; optional sweetness/acidity/intensity notes; preparation instructions and presentation notes.

Ingredients include gin, vodka, whiskey, rum, tequila, liqueurs, wine, syrups, soda, juice, other beverages. Configure ABV at ingredient or specific product level. Calculate final estimated ABV only with sufficient valid inputs, account for known mixers/dilution, and expose assumptions. Unknown ABV/dilution must not yield an exact claim; uncertain results labeled estimates, or unavailable when insufficient data.

## R08 — Ingredient catalog

Ship an extensive validated built-in catalog available immediately after installation, offline. Investigate USDA FoodData Central, FoodOn, Open Food Facts, and legally usable beverage references; verify licenses/reuse before external material is imported. Normalize and clean into canonical ingredients rather than dumping raw source databases. Include common Turkish ingredients and culinary terminology.

Coverage: vegetables; fruits; herbs/spices; grains; legumes; meat/poultry; seafood; dairy; eggs; oils/fats; sauces/condiments; sweeteners; baking ingredients; nuts/seeds; alcoholic ingredients; non-alcoholic liquids; coffee/tea ingredients; syrups/mixers; garnishes; specialty and user-defined ingredients.

Ingredient fields: stable unique ID, canonical name, Turkish display name, English name, alternatives/aliases, parent/child categories, type, preferred unit, allowed dimensions, optional density, nutrition, allergens, dietary metadata, alcohol percentage, product brand/barcode, provenance/license, user-defined metadata. Users create/edit/search/organize personal ingredients. Autocomplete, typo tolerance, duplicate detection. No fabricated names, nutrients, or provenance to inflate size. Coverage report must count actual successfully imported/validated ingredients per category. Built-in/imported updates never silently overwrite personal edits.

## R09 — Measurement and conversion engine

Configurable units:

- Volume: mL, cc, cL, dL, L, US fluid ounces, appropriate culinary units.
- Mass: mg, g, kg, oz, lb.
- Count: pieces, slices, portions, packets, pinches, drops, custom units.
- Culinary: teaspoons, dessert spoons, tablespoons, cups, configurable household measures.
- Bar: shot, jigger, dash, bar spoon, custom spirit measures.
- Temperature: Celsius/Fahrenheit.

Allow 25 cc, 35 cc, 50 cc, custom drink quantities; personal presets and preferred display units. Separate normalized quantity from display unit. Accurate compatible conversions; 1 cc = 1 mL, never assume 1 mL = 1 g. Cross mass/volume requires valid density. Precise calculations, tested correctness, scaling without destructive rounding, sensible display rounding and meaningful fractional count handling. Explain unsafe/unavailable conversions.

## R10 — Steps and Cooking Mode

Ordered steps with title/description, optional duration, temperature, equipment, images, ingredients used, technique, notes/tips. Distraction-free Cooking Mode with large readable instructions and step navigation.

## R11 — Pantry/inventory

Owned ingredients with quantities, units, locations, expiration dates, optional purchase prices. Multiple physical stock entries per ingredient definition. Calculate recipe availability/shortages; separate stock identity from canonical ingredient identity.

## R12 — Shopping lists

Generate from one/multiple recipes; aggregate ingredients, convert quantities, deduct stock, allow manual additions and completion, logical grouping. Explain incompatible quantities rather than silently combining them.

## R13 — Meal planning

Optional-use daily/weekly planner assigning recipes to days/meals, connected to portions, pantry, shopping lists. Included in V1.

## R14 — Collections/personal organization

Favorites, custom collections, cooking history, saved filters, private notes, ratings, recipes to try. Many-to-many collection membership.

## R15 — Statistics

Real database-backed meaningful recipe, category, ingredient, preparation, and collection statistics; no hardcoded demos.

## R16 — Database

Normalized SQLite entities/relationships: recipes, versions/variations, steps, recipe ingredients, ingredients, aliases, ingredient categories, categories, tags, recipe tags, measurements/units, presets, conversion metadata, media, collections/membership, ratings/preparation history, pantry, shopping lists/items, meal plans, preferences, external sources, seed versions/import records, schema migrations. Appropriate keys, foreign keys, indexes, constraints, cascades; related writes transactional. Repeatable versioned idempotent seeds; duplicate prevention and user-data protection.

## R17 — Media

Recipe/step photos in appropriate app-data directories with DB references; efficient thumbnails, no unnecessary full-resolution loading. Safe media references during deletion. Architecture ready for optional video references without hosting requirement.

## R18 — Local data, portability, safety

Personal data stays on the user's computer by default. Full portable backup includes database, related media, manifest, schema information, integrity metadata. SQLite transaction/WAL-safe backup. Validated restore with preview/confirmation before replacement; JSON recipes; appropriate tabular CSV import/export; interrupted/failed-import recovery; schema compatibility checks; human-readable errors. Backups transferable between macOS/Windows; no absolute machine paths in exports.

Offline; no mandatory accounts/default cloud; no telemetry without explicit consent. Input validation, secure filesystem permissions, SQL injection prevention, graceful corrupt/missing-data handling, accidental permanent-deletion protection, upgrade preservation, referential integrity, explicit import conflicts. Unknown nutrition differs from zero; missing allergen metadata never implies allergen-free.

## R19 — Repository and documentation

Maintain README.md, AGENTS.md, architecture, database schema, data dictionary, measurement specification, external-data/licensing docs, roadmap, detailed feature checklist, testing docs, CHANGELOG.md, progress/handoff, release instructions, backup/restore docs. Persistent progress records completed work, unresolved issues, next tasks, verification across AI sessions/tools.

## R20 — Quality, release, execution

Automated critical tests: recipe/ingredient CRUD; category relationships; autocomplete/search; Turkish characters; deduplication; conversions/scaling; cocktail quantities/estimated ABV; variations; pantry; shopping aggregation; migrations; seed idempotency; backup/restore; import validation; upgrade preservation; cross-platform filesystem. Meaningful UI tests; mocked implementation never proves unfinished functionality complete. Relevant type/lint/tests/production builds, explicit passed/failed/untested reports.

One Git repository; semantic versions, tags, GitHub Releases with historical installable versions. GitHub Actions builds macOS Apple Silicon, Intel where supported, Windows installer. Explicit signing/notarization; unsigned is not trusted signed release. Tag v1.0.0 only after agreed V1 criteria pass; keep V1 retrievable/installable. Document migration/downgrade/restore limits.

Execution sequence: preserve/inspect starter; actual structure; architecture/schema; working shell; persistent recipe workflow; ingredient/category catalogs; reliable seeding/importers; measurement; specialized food/beverage; integrated modules; backup/restore; tests/fixes; packaging/validation; release/docs. Every milestone implements real behavior, verifies it, leaves app runnable, updates docs/progress, commits logical Git changes, reports unfinished scope, resumes last verified state. Never mark incomplete acceptance criteria complete. The detailed plan may move foundational measurement/storage safety work earlier to satisfy dependencies.

## Open product decisions, not scope reductions

“Extensive” needs a reviewed coverage threshold and Turkish/beverage representative checklist before catalog acceptance; do not invent a guaranteed count. Choose minimum OS versions, rich-note format, rating scale, stock-consumption confirmation behavior, version-retention policy, and household unit defaults during their owning milestones. Until resolved, retain these as explicit gates. No chosen default may delete a required capability.
