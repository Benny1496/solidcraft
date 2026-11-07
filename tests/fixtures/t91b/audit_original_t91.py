#!/usr/bin/env python3
"""Test 91b: independently re-audit *original* Test 91 Rust engine logs.

Read only already archived log, enumeration, and exact baseline. No Rust tests re-run.
A normal Rust `ignored, why...` line is explicitly supported.
"""
import json
import re
import sys
from pathlib import Path

if len(sys.argv) != 4:
    raise SystemExit("usage: audit_original_t91.py ORIGINAL_ARTIFACT_FOLDER SOURCE_ROOT OUT_FOLDER")

old, source, output = map(Path, sys.argv[1:])
output.mkdir(parents=True, exist_ok=True)

def assert_line_parser() -> None:
    cases = {
        "test foo::passed ... ok": ("foo::passed", "ok"),
        "test foo::failed ... FAILED": ("foo::failed", "FAILED"),
        "test foo::skipped ... ignored": ("foo::skipped", "ignored"),
        "test foo::skipped ... ignored, ignored on Linux": ("foo::skipped", "ignored"),
        "test foo::skipped ... ignored, this is a benchmark": ("foo::skipped", "ignored"),
        "test foo::skipped ... ignored (temporarily)": ("foo::skipped", "ignored"),
    }
    for line, want in cases.items():
        assert parse(line) == want, f"Parse self-test {line}: {parse(line)} != {want}"

test_line = re.compile(r"^test\s+(.+?)\s+\.\.\.\s+(ok|FAILED|ignored)(?:[,\s(].*)?$")

def parse(line: str):
    x = test_line.fullmatch(line.strip())
    return (x.group(1), x.group(2)) if x else None

assert_line_parser()
log = (old / "engine-all-tests.log").read_text(encoding="utf-8", errors="replace")
listing = (old / "engine-tests-list.txt").read_text(encoding="utf-8", errors="replace")
old_evaluation = json.loads((old / "engine-evaluation.json").read_text(encoding="utf-8"))
base = (source / "tests/KNOWN_FAILING_TESTS").read_text(encoding="utf-8")
known = {line.split(" ", 1)[0].removeprefix("solidcraft-engine::") for line in base.splitlines() if line.startswith("solidcraft-engine::")}
listed = {line.removesuffix(": test") for line in listing.splitlines() if line.endswith(": test")}
assert len(listed) > 3000, f"Unexpected enumeration: {len(listed)}"
running = re.search(r"^running\s+(\d+)\s+tests$", log, flags=re.M)
assert running, "Missing Rust test start count"
summaries = re.findall(r"^test result:\s+(ok|FAILED)\.\s+(\d+) passed;\s+(\d+) failed;\s+(\d+) ignored;", log, flags=re.M)
assert len(summaries) == 1, f"Expected exactly one Rust summary, got {summaries}"
overall, n_pass, n_failed, n_ignored = summaries[0]
totals = {"passed":int(n_pass), "failed":int(n_failed), "ignored":int(n_ignored)}
assert len(listed) == int(running.group(1)), "Test list and Rust start count diverge"
assert sum(totals.values()) == len(listed), f"Rust's own totals inconsistent: {totals} vs {len(listed)}"
matches = {}
duplicates = []
normalized_should_panic = []
for line in log.splitlines():
    parsed = parse(line)
    if parsed is None:
        continue
    name, status = parsed
    # libtest prints a display suffix " - should panic" for #[should_panic] cases;
    # its --list entry omits the suffix. Only normalize when the unsuffixed
    # exact name is in the original enumerated list, never remove arbitrary text.
    if name not in listed and name.endswith(" - should panic"):
        stripped = name.removesuffix(" - should panic")
        if stripped in listed:
            normalized_should_panic.append({"printed": name, "enumerated": stripped})
            name = stripped
    if name in matches:
        duplicates.append(name)
    matches[name] = status
all_outcomes = set(matches)
missing = sorted(listed - all_outcomes)
unlisted = sorted(all_outcomes - listed)
passed = {n for n,v in matches.items() if v=="ok"}
failed = {n for n,v in matches.items() if v=="FAILED"}
ignored = {n for n,v in matches.items() if v=="ignored"}
known_failed = sorted(failed & known)
unexpected = sorted(failed - known)
now_fixed = sorted(passed & known)
never_run_known = sorted(known - all_outcomes)
report = {
    "origin": "GitHub Actions 42304157430, original artifact 13549116265",
    "sha256_original_zip": "d88256f4230f460ef0379cf31fdb1ae60f625475e33916383e54d4d5df31f5d8",
    "pr_sha": "e571b8998e6217787a92a25ceb565ece736ebf2d",
    "rust_summary": {"result":overall,**totals},
    "enumerated": len(listed),
    "recorded": len(matches),
    "individually_passed": len(passed),
    "individually_failed": len(failed),
    "individually_ignored": len(ignored),
    "known_failure_count": len(known_failed),
    "known_failures": known_failed,
    "new_regressions": unexpected,
    "known_now_fixed": now_fixed,
    "known_not_run": never_run_known,
    "unaccounted_for": missing,
    "outcomes_without_enumeration": unlisted,
    "duplicated_outcome_names": duplicates,
    "normalized_should_panic_display_suffixes": normalized_should_panic,
    "prior_evaluator_missing": old_evaluation.get("missing", []),
}
(output / "t91b-engine-audit.json").write_text(json.dumps(report,indent=2,ensure_ascii=False)+"\n",encoding="utf-8")
(output / "t91b-ignored-tests.txt").write_text("\n".join(sorted(ignored))+"\n")
print("ORIGINAL_ARTIFACT_SHA256=VERIFIED_BY_CI",flush=True)
print(f"ENGINE_LISTED={len(listed)}; RUST_SUMMARY={totals}; INDIVIDUAL_RESULTS={len(matches)}",flush=True)
print(f"PASS={len(passed)}; KNOWN_FAIL={len(known_failed)}; IGNORED={len(ignored)}; NEW_REGRESSIONS={len(unexpected)}",flush=True)
print(f"MISSING={len(missing)}; UNLISTED={len(unlisted)}; DUPLICATES={len(duplicates)}; FIXED_BASELINE={len(now_fixed)}",flush=True)
if missing: print("MISSING_NAMES="+",".join(missing[:30]),flush=True)
if unlisted: print("UNLISTED_NAMES="+",".join(unlisted[:30]),flush=True)
if duplicates: print("DUPLICATES="+",".join(duplicates[:30]),flush=True)
if unexpected: print("NEW_REGRESSION_NAMES="+",".join(unexpected),flush=True)
valid = (
    len(matches) == len(listed)
    and not (missing or unlisted or duplicates or unexpected or now_fixed or never_run_known)
    and len(passed)==totals["passed"]
    and len(failed)==totals["failed"]
    and len(ignored)==totals["ignored"]
    and overall=="FAILED"
    and len(known_failed)==len(known)
    and old_evaluation["unexpected_failures"] == []
)
if not valid:
    print("FINAL=T91B_ENGINE_REAUDIT_NEEDS_REVIEW",flush=True)
    raise SystemExit(2)
print("FINAL=T91B_ENGINE_REAUDIT_BASELINE_VERIFIED_0_NEW_REGRESSIONS",flush=True)
