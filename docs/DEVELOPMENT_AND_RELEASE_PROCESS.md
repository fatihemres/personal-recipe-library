# Development branches, CI, and release safety

## Branch and PR policy

`origin/main` is the integration branch. Each milestone starts from a freshly fetched, verified `main` and uses a dedicated feature branch. The one-time `feature/m3d-2b-release8-ci` branch starts at local release-8 HEAD and carries the existing M3D-2A and M3D-2B commits unchanged. Its merge base is release-7 commit `0e7f985`; no reset, cherry-pick, or duplicate milestone commit is needed.

For subsequent milestones:

1. Check for local edits, especially untracked personal files. Fetch `origin`, verify the PR merge completed, and fast-forward local `main` with `git merge --ff-only origin/main`. Never reset or discard local work to make this command pass.
2. Create a new feature branch from that updated `main`. Keep the change within its approved milestone and preserve applied migrations/catalog releases.
3. Run the local checks in `docs/TESTING.md`, review the diff, and commit scoped changes. Push only the feature branch and open a PR to `main`.
4. Inspect all GitHub checks and downloadable artifacts. Repair real failures on the same branch; do not suppress checks. Report native GUI and real-device verification separately.
5. Obtain explicit Product Owner review before merging. Use a **merge commit** for this transition PR and for subsequent milestone PRs so original milestone commit ancestry remains intact. Do not use squash/rebase merge for the transition. Do not enable auto-merge.
6. Once merged and verified, fetch and fast-forward local `main`. Add a unique annotated milestone checkpoint tag only to the verified merged commit, if the milestone calls for one. Never overwrite a tag or delete a branch containing unmerged work.

The private repository currently returns HTTP 403 for branch-protection configuration with a message requiring GitHub Pro or public visibility. Keep it private and apply a manual **no-direct-main-push, PR-and-checks-before-merge** policy. If the plan later supports protection, first observe actual successful CI context names, then require PRs, those exact passing contexts, resolved conversations, and disallow force pushes/deletion and unreviewed bypass. Do not make an untested check required or change plans/visibility as a workaround.

## PR validation

`.github/workflows/pr-ci.yml` runs only for PRs targeting `main`. Two Ubuntu jobs independently check frontend/browser and Rust/SQLite/catalog behavior. The three platform packaging jobs wait for both fast jobs. Superseded runs for the same PR are cancelled. The workflow has read-only repository permission, pinned Node 24.21.0 and Rust 1.99.0 toolchains, npm/Cargo caches, bounded timeouts, no schedule, and seven-day artifacts. It uses standard GitHub-hosted runners: Ubuntu 24.04, macOS 15 arm64, macOS 15 Intel, and Windows 2025 x64. GitHub-hosted minutes and artifact storage for a private repository count against the account allowance; inspect billing before repeatedly rebuilding. No paid larger runner or secrets are configured.

The jobs upload **unsigned PR test artifacts**: ad-hoc-signed macOS `.app` ZIP and DMG for each architecture, and a Windows NSIS setup executable. Each artifact includes SHA-256 hashes. Verify the checksum after download. Successful compilation proves the source built on that runner; successful packaging proves an artifact exists. Browser and SQLite tests do not prove a native GUI workflow, a clean-install experience, or trusted signing/notarization. A downloaded unsigned/ad-hoc package can trigger platform trust warnings. A public release requires separate signing/notarization, installer, upgrade, and native manual tests.

Runner labels and architecture were checked against [GitHub's hosted-runner reference](https://docs.github.com/en/actions/reference/runners/github-hosted-runners); platform prerequisites and package types against [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) and [Windows installer guidance](https://v2.tauri.app/distribute/windows-installer/). These upstream references can change, so recheck them when updating runner images or targets.

## Version identities and recovery

Feature branches identify work in progress; commits are immutable code/data snapshots. Annotated `milestone/<name>` checkpoint tags identify **merged, verified** commits and are never moved. Catalog release numbers are separately versioned SQLite seed packages: release 8 is in `catalog/production`, release 7 remains in `catalog/releases/7`. They are not application versions. The current application manifests say `0.1.0`, which is pre-1.0 development; do not tag or advertise `v1.0.0` until all V1 acceptance criteria pass. GitHub Releases are separate, reviewed distributions of signed artifacts and their checksums; no PR CI job creates a Release or tag.
The Turkish sidebar and Settings/About view read the installed application version from Tauri package metadata. `tools/check_app_version.py` checks the npm, Cargo and Tauri manifests during PR CI; catalog release numbering is never used as the application version.

To recover from a bad code merge, preserve the merged history and use a reviewed `git revert` PR or a new fix PR; retain the old commit and checkpoint tag. Reverting Git code **does not undo SQLite migrations, catalog imports, or changes to user data**. Before a risky application upgrade, take and validate a portable backup using the application's backup mechanism when it exists; until then, follow the database-copy/WAL safety guidance in `docs/DATABASE_SCHEMA.md` and avoid opening a newer database with an older application. Restore a compatible database/media backup separately from code recovery. Never use `git reset --hard`, force-push, or a destructive rebase as a production rollback procedure.
