# M3D-4 isolated native and performance verification — 2026-10-19

An unsigned **production** arm64 Tauri `.app` was built with the temporary identifier `com.recipeatlas.m3d4-verification` and app-data override `/private/tmp/recipeatlas-m3d4-native-profile`. The normal application identifier and Product Owner profile were not used. The bundle launched with macOS LaunchServices outside the nested command sandbox; the sandboxed `open` returned `kLSNoExecutableErr` before launch, consistent with earlier sandbox limits.

The native process created SQLite schema 5, installed catalog Release 10 exactly once and exposed 795 canonical identities, 269 aliases and 50 categories. After `SIGTERM` and a complete new process launch, the same isolated profile still had one successful Release-10 import, `PRAGMA integrity_check=ok` and no foreign-key violations. The test process exited after verification. The bundled Rust/SQLite regression separately exercises Turkish/English lookup, category membership, recipe references, exact decimal quantities, draft, favorite and override preservation.

The computer-use inventory saw the running `com.recipeatlas.m3d4-verification` process, but its native window binding attempted the stale `com.recipeatlas.m3d3-verification` ID and then rejected the current ID. Therefore **no Release-10 GUI clicks, ingredient selection or recipe-edit actions are claimed**. The Product Owner should test those in a downloadable PR artifact. Windows and Intel native GUI/installer behavior, signing and notarization remain unverified locally.

For bounded resource context, one native process was observed at 75,488 KiB RSS after about 29 seconds and 68,192 KiB after about 73 seconds idle, with 17 thread rows and 62 `lsof` lines in the latter sample. These two samples do not establish a leak trend or a controlled Release-9 comparison. The earlier DEVOPS-1C lifecycle audit remains the deeper baseline.

The real SQLite regression measured 100 Turkish partial searches in 161.957 ms in a local debug test process. The Release-10 importer measured isolated upgrade wall times including debug CLI startup of 333.929 ms (Release 7), 335.791 ms (Release 8), and 343.163 ms (Release 9). These are local single-run observations, not service-level guarantees or a controlled improvement claim. A process-local macOS network-denial wrapper completed 26 fresh/historical/repeat import opens; an initial invocation inside the nested tool sandbox exited 71 before any import and is not counted as a pass.

## Product Owner arm64 acceptance checklist

Download `unsigned-macos-arm64` from [PR #3 Actions run 39214465248](https://github.com/fatihemres/personal-recipe-library/actions/runs/39214465248), verify the included `SHA256SUMS.txt`, and use the app on the Mac where manual acceptance is intended. The package is unsigned/ad-hoc signed, not a notarized public release. Avoid deleting or resetting an existing profile while testing.

1. Launch the app and confirm there is no catalog or local-storage error.
2. Open Malzeme Kütüphanesi; confirm 795 built-in identities on a clean profile, or 796 visible on the known historical preview profile with its retained extra identity. Personal ingredients are counted separately.
3. Search a newly added ingredient by its Turkish and English names (for example `Tulum peyniri` / `Tulum cheese`) and open its details; check the source display.
4. Browse the Turkish-regional category and a global category, confirming new entries appear without duplicate search rows.
5. Create a recipe using a newly added catalog ingredient; enter a distinctive decimal quantity and selected unit, then save and reopen it.
6. Edit that recipe, cancel an unsaved change, and confirm the last saved quantity remains exact.
7. Fully quit the application and relaunch it; confirm the ingredient, recipe and quantity remain available and startup is error-free.
8. On an existing profile, confirm pre-existing personal ingredients, recipes, drafts, favorites and presentation overrides remain intact. Do not discard a draft merely to complete this check.

These GUI and real-device checks are **pending Product Owner execution**. The five PR jobs passed and uploaded all three unsigned platform artifacts; their checksum manifests were verified after download. CI package generation does not establish GUI behavior on Intel macOS or Windows.
