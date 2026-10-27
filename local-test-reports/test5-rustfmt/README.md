# Archived Rustfmt correction — targeted test #5

The exact formatting patch is saved here as a **separate, unapplied archive**. This
branch starts at the already tested Atomic Undo/Redo correction commit and does
**not** apply the Rustfmt changes to the five production/source files.

## Provenance

- Source: GitHub Actions run [40452123990](https://github.com/Benny1496/ESP32-Victron-SolarSense-Bridge/actions/runs/40452123990)
- Job: `Verify formatting and export exact rustfmt patch`
- Generated with `cargo fmt --all` using **Rust/rustfmt 1.99.1**
- Source branch: `fix/test-5-atomic-roundtrip`
- Source commit: `a8633b3d550d8ce8b6acf22445912964a3793658`
- Patch file: [rustfmt-1.99.1.patch](rustfmt-1.99.1.patch)
- SHA-256: `9ff344f92decde1861c24d7552a6707a5d26a0ef4d67479f0cb0eea995ae1194`
- Bytes (UTF-8): **212574**
- Git blob SHA-1: `57a64dfecc396acb1afe106941b1bbe23df2d36c`
- Patch coverage: **5** files, **1629 insertions / 1253 deletions**.

## Changed file paths recorded in the patch

- `crates/engine/src/cmd/drawing_preview.rs`
- `crates/engine/src/cmd/edit_tests.rs`
- `crates/ui-egui/src/drawing_ui.rs`
- `crates/ui-egui/src/lib.rs`
- `xtask/src/web_offline_test.rs`

## Validation of generated patch

The Actions worker successfully completed `cargo fmt --all`,
`git diff --check` and a second `cargo fmt --all -- --check`.
The formatting job was then *intentionally marked failed* because the
unformatted **input branch** failed its initial check.

The independent Atomic Undo/Redo job finished **3 passed / 0 failed**.

## Reuse

Review before application. The patch was generated against the exact source
commit above, not against current upstream `main`. For a suitable clean
checkout of that base or a compatible descendant:

```bash
git apply --check local-test-reports/test5-rustfmt/rustfmt-1.99.1.patch
git apply local-test-reports/test5-rustfmt/rustfmt-1.99.1.patch
cargo fmt --all -- --check
```

First create a **new review branch** and inspect the diff. Do not apply the
patch directly to the preserved base branch or the official SolidCraft branches.
Re-run the complete tests after integrating the formatted sources.

This archival branch preserves a reproducible input patch beyond the original
GitHub Actions artifact's seven-day retention period.
