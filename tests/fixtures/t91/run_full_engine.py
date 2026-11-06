#!/usr/bin/env python3
"""Run the entire precompiled solidcraft-engine libtest binary and classify known failures.

Does not silently skip tests. The official tests/KNOWN_FAILING_TESTS is the baseline.
Test executables come from cargo test --no-run --message-format=json.
"""
import json
import re
import subprocess
import sys
from pathlib import Path

if len(sys.argv) != 4:
    raise SystemExit("usage: full_engine.py build-artifacts.jsonl report-dir source-root")
buildlog, report_dir, source_root = map(Path, sys.argv[1:])
report_dir.mkdir(parents=True, exist_ok=True)
source_root = source_root.resolve()
assert (source_root / "tests/KNOWN_FAILING_TESTS").is_file()
expected_failures = set()
for line in (source_root / "tests/KNOWN_FAILING_TESTS").read_text().splitlines():
    if line.startswith("solidcraft-engine::"):
        expected_failures.add(line.split(" ", 1)[0].removeprefix("solidcraft-engine::"))
engine_bins = []
for line in buildlog.read_text().splitlines():
    try:
        artifact = json.loads(line)
    except ValueError:
        continue
    if artifact.get("reason") != "compiler-artifact":
        continue
    target = artifact.get("target", {})
    binary = artifact.get("executable")
    if binary and target.get("name") == "solidcraft_engine" and "lib" in target.get("kind", []):
        engine_bins.append(binary)
engine_bins = sorted(set(engine_bins))
assert len(engine_bins) == 1, f"Expected one compiled engine lib-test executable, got {engine_bins}"
binary = engine_bins[0]
dir_engine = source_root / "crates/engine"
listing = subprocess.run([binary, "--list"], cwd=dir_engine, text=True, capture_output=True, timeout=150)
assert listing.returncode == 0, "Test enumeration failed: " + listing.stderr[-1000:]
listed = [x[:-6] for x in listing.stdout.splitlines() if x.endswith(": test")]
assert len(listed) > 3000, f"Too few compiled engine tests: {len(listed)}"
(report_dir / "engine-tests-list.txt").write_text(listing.stdout)
print(f"ENGINE_LISTED={len(listed)}; EXPECTED_KNOWN_FAILURES={len(expected_failures)}", flush=True)

output_file = report_dir / "engine-all-tests.log"
with output_file.open("w") as output:
    try:
        completed = subprocess.run(
            [binary, "--test-threads=2", "--nocapture"],
            cwd=dir_engine, text=True, stdout=output, stderr=subprocess.STDOUT, timeout=65 * 60
        )
        code = completed.returncode
    except subprocess.TimeoutExpired:
        print("FINAL=ENGINE_FULL_SUITE_TIMEOUT", flush=True)
        raise SystemExit(3)

text = output_file.read_text(encoding="utf-8", errors="replace")
results = {}
for line in text.splitlines():
    match = re.match(r"^test (.+?) \.\.\. (ok|FAILED|ignored)(?: .*)?$", line.strip())
    if match:
        results[match.group(1)] = match.group(2)
passed = {k for k, v in results.items() if v == "ok"}
failed = {k for k, v in results.items() if v == "FAILED"}
ignored = {k for k, v in results.items() if v == "ignored"}
unexpected = sorted(failed - expected_failures)
known = sorted(failed & expected_failures)
fixed = sorted(passed & expected_failures)
missing = sorted(set(listed) - set(results))
sums = re.findall(r"test result: (ok|FAILED)\.\s+(\d+) passed;\s+(\d+) failed;\s+(\d+) ignored", text)
data = {
    "binary": binary, "listed": len(listed), "recorded": len(results),
    "passed": len(passed), "failed": len(failed), "ignored": len(ignored),
    "known_failures": known, "unexpected_failures": unexpected,
    "baseline_fixed": fixed, "missing": missing,
    "subprocess_returncode": code, "summaries": sums,
}
(report_dir / "engine-evaluation.json").write_text(json.dumps(data, indent=2) + "\n")
print("ENGINE_TOTAL={listed}; RECORDED={recorded}; PASS={passed}; FAIL={failed}; IGNORED={ignored}".format(**data), flush=True)
print("KNOWN_FAILURES=" + str(len(known)) + "; NEW_REGRESSIONS=" + str(len(unexpected)) + "; FIXED_BASELINE=" + str(len(fixed)) + "; MISSING=" + str(len(missing)), flush=True)
if unexpected:
    print("UNEXPECTED_FAILURE_NAMES=" + ", ".join(unexpected), flush=True)
if missing:
    print("MISSING_TEST_NAMES=" + ", ".join(missing[:30]), flush=True)
if fixed:
    print("FIXED_KNOWN_FAILURES=" + ", ".join(fixed), flush=True)
if not sums or len(results) < len(listed) or missing or unexpected:
    print("FINAL=ENGINE_UNEXPECTED_REGRESSION_OR_INCOMPLETE", flush=True)
    raise SystemExit(2)
if len(failed) > 0 and code == 0 or len(failed) == 0 and code != 0:
    print("FINAL=ENGINE_INCONSISTENT_TEST_PROCESS", flush=True)
    raise SystemExit(4)
if fixed:
    print("FINAL=KNOWN_FAILURE_BASELINE_STALE_NEEDS_UPDATE", flush=True)
    raise SystemExit(5)
print("FINAL=ENGINE_SUITE_BASELINE_VALID; KNOWN_FAILURES=" + str(len(known)), flush=True)
