# SolidCraft Test 86 – latest drawings engine integration

This fork-only four-way native Rust check pins upstream `drawings@e859b77` (includes merged #12573). It reuses already-tested patches from Test 85 and does not alter either official PR.

Matrix: upstream bare, #8288 only, #11881 only, and #11881 + #8288 together. The workflow first requires patch application to succeed, then runs repository/static checks, Rust clippy, targeted drawing/reopen/deferred-update tests and the entire engine nextest suite with upstream known-failing classification. It preserves logs as artifacts.

Important: `git apply --3way` may reveal genuine integration conflicts when older PR patches meet the newer base; this is **not** a claimed test pass. Any setup, patch or compile failure must be reported before interpreting engine results.

The two known #8288 reference-ID failures are expected **only if** they match the explicit upstream known-failing list; a new regression is not accepted. This workflow does not perform the complete visual UI/paper E2E acceptance. No #12394 hidden-balloon refusal test is included because that PR has not merged.

Start: push of `.github/workflows/solidcraft-w23a-update-86.yml` to the fork-only test branch. Do not dispatch `ci.yml` on main.
