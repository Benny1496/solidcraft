#!/usr/bin/env python3
"""T105: execute the full native engine library suite, excluding ONLY the
upstream documented known-failing tests. Logs all test output, fails on any
other regression, and reports which known failures were skipped.

This is not a green run of the excluded cases; they are separate known issues.
"""
import json
import re
import subprocess
import sys
from pathlib import Path

if len(sys.argv) != 4:
    raise SystemExit("usage: run_engine_except_known.py cargo-artifacts-jsonl output-log output-summary")
artifacts, outlog, summarypath = map(Path, sys.argv[1:])
rows = artifacts.read_text(encoding="utf-8").splitlines()
matches = []
for line in rows:
    try:
        e = json.loads(line)
    except ValueError:
        continue
    if e.get("reason") == "compiler-artifact" and e.get("target",{}).get("name")=="solidcraft_engine" and "lib" in e.get("target",{}).get("kind",[]) and e.get("executable"):
        matches.append(e["executable"])
matches = sorted(set(matches))
assert len(matches)==1, f"engine test lib executable must be unique: {matches}"
binary = matches[0]
listing = subprocess.run([binary,"--list"],capture_output=True,text=True,timeout=120,cwd="crates/engine")
assert listing.returncode == 0, listing.stderr
alltests = [s[:-6] for s in listing.stdout.splitlines() if s.endswith(": test")]
known_lines=Path("tests/KNOWN_FAILING_TESTS").read_text(encoding="utf-8").splitlines()
known = [x.split(" ",1)[0][len("solidcraft-engine::"):] for x in known_lines if x.startswith("solidcraft-engine::")]
found = sorted({name for name in known if name in alltests})
unresolved = sorted(set(known)-set(found))
assert len(alltests)>=3500, f"expected broad engine suite, got {len(alltests)}"
assert len(found)>=8, f"expected at least eight upstream documented failures in engine, saw {found}; unresolved={unresolved}"
cmd=[binary,"--nocapture"]
for name in found:
    cmd.extend(["--skip",name])
result=subprocess.run(cmd,capture_output=True,text=True,timeout=1800,cwd="crates/engine")
log=result.stdout+"\n"+result.stderr
outlog.write_text(log,encoding="utf-8")
m=re.search(r"test result: ok\.\s+(\d+) passed;\s+(\d+) failed;",log)
summary={
    "binary":binary,"total_discovered":len(alltests),
    "excluded_documented_known_failures":found,"known_list_names_not_found":unresolved,
    "tested_pass":int(m.group(1)) if m else -1,
    "tested_fail":int(m.group(2)) if m else -1,
    "exit_code":result.returncode,
    "note":"Skipped documented known failures are NOT counted as successful tests."
}
summarypath.write_text(json.dumps(summary,indent=2),encoding="utf-8")
print(json.dumps(summary,indent=2),flush=True)
if result.returncode!=0 or not m or int(m.group(2)) or int(m.group(1))<3400:
    print(log[-12000:],file=sys.stderr)
    raise SystemExit("Engine full suite has new failures or too few passed tests")
