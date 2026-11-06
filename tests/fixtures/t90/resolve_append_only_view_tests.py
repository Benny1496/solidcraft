#!/usr/bin/env python3
"""Resolve the single verified *append/append* W23 view-test merge conflict.

Strictly for a temporary CI checkout. Abort on any unexpected change.
Keep upstream's entire current file, then add PR #11881's self-contained
appended W23 helper and test. Never alter the official repository.
"""
from pathlib import Path
import subprocess

PATH = "crates/engine/src/cmd/drawing_view_tests.rs"
BASE = "67a325808f5748e52b254849d7aef4b41ecfdce7"
PR = "62c1fbe95173c00878fb733f9a5f2b32f39eb560"

def git(*args: str) -> str:
    return subprocess.check_output(["git", *args], text=True).strip()

conflicts = git("diff", "--name-only", "--diff-filter=U").splitlines()
assert conflicts == [PATH], f"Refuse unexpected conflict list: {conflicts}"
merge_base = git("merge-base", BASE, PR)
assert merge_base == "855ce28fda9efe46f8a71c581a1be825604e4a9b", (
    f"Unexpected common ancestor {merge_base}"
)

def show(rev: str) -> bytes:
    return subprocess.check_output(["git", "show", f"{rev}:{PATH}"])

ancestor = show(merge_base)
incoming = show(PR)
upstream = show(BASE)
assert incoming.startswith(ancestor), (
    "PR #11881 changed older view-test lines, not only appended W23 block"
)
added = incoming[len(ancestor):]
assert added.startswith(b"\n/// The W23 assembly sheet's assembly (#11773)"), (
    "Unexpected W23-only appendix in PR #11881"
)
assert added.count(b"fn assembly_sheet()") == 1
assert added.count(b"fn the_assembly_sheets_views_keep_what_they_show_through_the_model()") == 1
assert not any(n in upstream for n in [
    b"fn the_assembly_sheets_views_keep_what_they_show_through_the_model()",
    b'include_str!("../../../../tests/fixtures/w23-assembly-sheet/assembly.json")',
])
assert upstream.endswith(b"\n"), "Official file does not have normal newline termination"
assert b"<<<<<<<" not in added and b">>>>>>>" not in added

result = upstream + added
assert upstream in result and added in result
Path(PATH).write_bytes(result)
subprocess.run(["git", "add", "--", PATH], check=True)
subprocess.run(["git", "diff", "--cached", "--check"], check=True)
assert not git("diff", "--name-only", "--diff-filter=U"), "Unmerged paths remain"
assert result.count(b"fn a_left_out_part_says_where_each_kind_of_setter_set_it()") == 1, (
    "Lost #12394 upstream view-test control"
)
print(f"RESOLUTION=PASS; MERGE_BASE={merge_base}; UPSTREAM_BYTES={len(upstream)}; W23_APPEND_BYTES={len(added)}; COMBINED_BYTES={len(result)}", flush=True)
