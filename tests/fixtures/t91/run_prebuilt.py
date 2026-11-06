#!/usr/bin/env python3
"""Execute only a previously compiled Rust unit-test binary.
Reads cargo test --no-run --message-format=json; does not invoke cargo again.
For the scratch case, an expected assertion failure is evidence of an
unfixed bug, not of a product fix.
"""
import json
import re
import subprocess
import sys
from pathlib import Path

if len(sys.argv) != 7:
    raise SystemExit("usage: run_prebuilt.py cargo_json crate filter report expected_kind minimum")
cargo_log, crate, needle, report_path, kind, minimum_raw = sys.argv[1:]
minimum = int(minimum_raw)
assert kind in ("pass", "known_scratch_fail")
events = Path(cargo_log).read_text(encoding="utf-8").splitlines()
executables = []
for line in events:
    try:
        event = json.loads(line)
    except (json.JSONDecodeError, ValueError):
        continue
    if event.get("reason") != "compiler-artifact":
        continue
    target = event.get("target", {})
    kind_list = target.get("kind", [])
    path = event.get("executable")
    if path and target.get("name") == crate and "lib" in kind_list:
        executables.append(path)
executables = sorted(set(executables))
if len(executables) != 1:
    raise SystemExit(f"Expected exactly one precompiled lib-test executable for {crate}; got {len(executables)}")

binary = executables[0]
crate_dir = {"solidcraft_engine": "crates/engine", "solidcraft_ui_egui": "crates/ui-egui"}.get(crate)
if crate_dir is None or not Path(crate_dir).is_dir():
    raise SystemExit(f"Unsupported test package or missing working directory: {crate}")
listed = subprocess.run([binary, "--list"], text=True, capture_output=True, timeout=90, cwd=crate_dir)
if listed.returncode != 0:
    raise SystemExit("Could not list executable's compiled tests: " + listed.stderr[-1200:])
matches = [line for line in listed.stdout.splitlines()
           if line.endswith(": test") and needle in line.split(": test")[0]]
if len(matches) < minimum:
    raise SystemExit(f"Expected >={minimum} compiled tests for {needle}, got {len(matches)}")
if kind == "known_scratch_fail" and len(matches) != 1:
    raise SystemExit(f"Scratch negative testcase must match exactly one test, found {len(matches)}")

print(f"PREBUILT={binary}", flush=True)
print(f"FILTER={needle}; MATCHES={len(matches)}; EXPECTATION={kind}", flush=True)
try:
    result = subprocess.run([binary, needle, "--nocapture"],
                            text=True, capture_output=True, timeout=900, cwd=crate_dir)
except subprocess.TimeoutExpired as exc:
    raise SystemExit(f"Precompiled Rust test execution timed out: {needle}") from exc
output = result.stdout + "\n" + result.stderr
Path(report_path).write_text(output, encoding="utf-8")
print(output[-4500:], flush=True)

if kind == "pass":
    m = re.search(r"test result: ok\.\s+(\d+) passed;\s+(\d+) failed;", output)
    if result.returncode != 0 or not m or int(m.group(1)) < minimum or int(m.group(2)):
        raise SystemExit(f"TARGET_TESTS_NOT_PASS: filter={needle}, code={result.returncode}")
    print(f"RESULT=PASS; PASSED={m.group(1)}; FAILED={m.group(2)}")
else:
    found_name = "a_held_deferred_drawing_exports_the_same_from_a_scratch_copy"
    if result.returncode == 101 and found_name in output and (
        "scratch background export should match synchronous export" in output
        or "assertion `left == right` failed" in output
    ) and re.search(r"test result: FAILED\.\s+0 passed;\s+1 failed;", output):
        print("RESULT=KNOWN_SCRATCH_EXPORT_BUG_REPRODUCED_NOT_FIXED")
    elif result.returncode == 0:
        raise SystemExit("UNEXPECTED_NEGATIVE_TEST_PASS: verify if upstream product bug was fixed")
    else:
        raise SystemExit("NEGATIVE_TEST_NOT_EVALUABLE: failed for a reason other than the expected export assertion")
