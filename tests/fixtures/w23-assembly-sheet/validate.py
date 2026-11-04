#!/usr/bin/env python3
"""Check the W23 assembly sheet recipes (#11773) against the assembly they start from.

The hand counts are worked out again here from assembly.json itself, the command
script the native tests run, so the recipes can't drift from the assembly. This
is a static check of a written contract: it runs no SolidCraft command and never
reports a recipe as passed.
"""
import json
import re
from collections import Counter
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]

# Commands that change the model, refused while a drawing is open.
MODEL_COMMANDS = {"assembly.bom.renumber", "selection.delete", "occurrence.move", "edit.undo", "edit.redo"}
IDS = ["W23-AS-03", "W23-AS-04", "W23-AS-05", "W23-AS-06"]


def require(ok, message):
    if not ok:
        raise ValueError(message)


def read(name):
    return json.loads((HERE / name).read_text(encoding="utf-8"))


class Assembly:
    """The placements assembly.json makes: which component sits how often in which parent."""

    def __init__(self, script):
        self.placed = {}  # name -> [(component, parent)] for every `as` step
        self.parents = {}  # component -> parent it was created in
        self.inside = Counter()  # (parent, component) -> placements
        self.part_numbers, self.descriptions = {}, {}
        for step in script["commands"]:
            command, p = step["command"], step["params"]
            if command == "component.create":
                parent = p.get("parent", "root")
                self.parents[p["name"]] = parent
                self.inside[(parent, p["name"])] += 1
                placed = (p["name"], parent)
            elif command == "occurrence.copy":
                parent = p.get("parent", "root")
                self.inside[(parent, p["component"])] += 1
                placed = (p["component"], parent)
            elif command == "component.metadata.set":
                self.part_numbers[p["component"]] = p["part_number"]
                self.descriptions[p["part_number"]] = p["description"]
                continue
            else:
                continue
            if "as" in step:
                self.placed[step["as"]] = placed

    def children(self, parent, without=Counter()):
        return Counter({c: n - without[(parent, c)] for (p, c), n in self.inside.items() if p == parent}) - Counter()

    def first_level(self, without=Counter()):
        return {self.part_numbers[c]: n for c, n in self.children("root", without).items()}

    def parts_only(self, without=Counter()):
        counts = Counter()

        def walk(component, times):
            inside = self.children(component, without)
            if not inside:
                counts[self.part_numbers[component]] += times
            for child, n in inside.items():
                walk(child, times * n)

        for child, n in self.children("root", without).items():
            walk(child, n)
        return dict(counts)


def main():
    data = read("cases.json")
    asm = Assembly(read(data["assembly"]))
    require(data["schema"] == "w23-assembly-sheet-acceptance/1", "unexpected schema")
    require(data["no_runtime_pass_claimed"] is True, "no recipe may claim a pass")
    cases = data["cases"]
    require([c["id"] for c in cases] == IDS, "recipes 3 to 6 of #11773, in order")
    require(all(c["runtime_status"] == "NOT_RUN" and c["waits_for"] for c in cases), "each recipe waits for what it needs")

    # The hand counts, worked out again from the script.
    hand = data["hand_counts"]
    second = Counter({asm.placed["second_holder"][::-1]: 1})
    require(asm.parts_only() == hand["parts_only"], f"parts only: {asm.parts_only()}")
    require(asm.first_level() == hand["first_level"], f"first level: {asm.first_level()}")
    require(asm.parts_only(second) == hand["second_holder_deleted"]["parts_only"], "parts only without the second holder")
    require(asm.first_level(second) == hand["second_holder_deleted"]["first_level"], "first level without the second holder")
    require(any(n > 1 for n in asm.parts_only().values()), "at least one part used more than once")

    # Every name a recipe uses is one the script gives.
    names = set(asm.placed)
    for case in cases:
        for word in re.findall(r'"(first_holder|second_holder|block|plate|screw)"', json.dumps(case)):
            require(word in names, f"{case['id']}: {word} is not named in the script")

    # Model edits only with the drawing closed, then the drawing opened again before a check.
    for case in cases:
        state = "closed"
        for step in case["steps"]:
            command = step.get("do")
            if command in MODEL_COMMANDS:
                require(state == "closed", f"{case['id']}: {command} with the drawing open")
            if "from" in step or command in ("drawing.create", "drawing.open"):
                # A recipe continued from another starts where that one left the drawing: open.
                state = "open"
            elif command in ("drawing.close", "doc.open"):
                state = "closed"
            if "drawing" in step:
                require(step["drawing"] == state, f"{case['id']}: {command or step.get('pending')} says the drawing is {step['drawing']}")
            if "check" in step:
                require(state == "open" or command == "drawing.export", f"{case['id']}: a check with the drawing closed")
                require(step["check"] in case["expect"], f"{case['id']}: no expected values for {step['check']}")

    # W23-AS-03: the rows are the first level, numbered 1, 2, 3 in part-number order, with the
    # script's descriptions; the delete changes only the holder's quantity.
    rows = cases[0]["expect"]
    for key, counts in (("rows", asm.first_level()), ("rows_after_delete", asm.first_level(second))):
        listed = rows[key]
        require([r["part_number"] for r in listed] == sorted(counts), f"W23-AS-03 {key}: part numbers")
        require([r["item"] for r in listed] == [str(i) for i in range(1, len(listed) + 1)], f"W23-AS-03 {key}: items")
        for r in listed:
            require(r["quantity"] == counts[r["part_number"]], f"W23-AS-03 {key}: {r['part_number']} quantity")
            require(r["description"] == asm.descriptions[r["part_number"]], f"W23-AS-03 {key}: {r['part_number']} description")

    # W23-AS-04: each balloon goes through its own holder to the block, shows the holder's item,
    # and the deleted holder's balloon is lost, never moved or removed.
    holder_item = next(r["item"] for r in rows["rows"] if r["part_number"] == asm.part_numbers["Holder"])
    balloons = cases[1]["expect"]
    for check in ("both_ok", "second_lost"):
        for name, holder in (("first_balloon", "first_holder"), ("second_balloon", "second_holder")):
            b = balloons[check][name]
            require(b["occurrence"] == [holder, "block"], f"W23-AS-04 {check}: {name} through its own holder")
            if b["status"] == "ok":
                require(b["item"] == holder_item, f"W23-AS-04 {check}: {name} shows item {holder_item}")
    lost = balloons["second_lost"]["second_balloon"]
    require(lost["status"] == "lost" and lost["moved_to_first_holder"] is False and lost["removed"] is False, "W23-AS-04: lost, never moved or removed")
    require(balloons["second_lost"]["first_balloon"]["status"] == "ok", "W23-AS-04: the first holder's balloon stays ok")

    # W23-AS-05: three revisions, the second deleted, the next is still one past the highest.
    rev = cases[2]["expect"]
    made = [s for s in cases[2]["steps"] if s.get("pending") == "revise"]
    require(rev["three_rows"]["revisions"] == [str(i) for i in range(1, len(made) + 1)], "W23-AS-05: numbered 1, 2, 3")
    left = [r for r in rev["three_rows"]["revisions"] if r != "2"]
    require(rev["after_delete"]["revisions"] == left and rev["after_delete"]["next"] == str(len(made) + 1), "W23-AS-05: 1 and 3, next 4")
    require(all(t["revision"] in left for t in rev["after_delete"]["tags"]), "W23-AS-05: a tag names a revision that is there")
    require(cases[2]["expect"]["drawing_record_unchanged_by_the_model_edit"] is True, "W23-AS-05: unchanged by the model")

    # W23-AS-06: the same bytes before saving and after opening again.
    export = cases[3]["expect"]["same_export"]
    require(export["bytes_equal"] == ["before", "after"] and export["formats_now"] == ["svg"], "W23-AS-06: same SVG bytes")

    # Everything placed on paper fits the sheet.
    width, height = data["sheet"]["paper_mm"]
    for at in re.findall(r'"(?:at_mm|position_mm)": \[([0-9.]+), ([0-9.]+)\]', json.dumps(data)):
        x, y = map(float, at)
        require(0 <= x <= width and 0 <= y <= height, f"{at} is off the A3 sheet")

    # The native tests the recipes rely on are there, and the README names every recipe.
    for file, tests in data["already_native"].items():
        text = (ROOT / file).read_text(encoding="utf-8")
        for name in tests:
            require(re.search(r"#\[test\]\s*\nfn " + name + r"\(\)", text), f"{file}: no test {name}")
    readme = (HERE / "README.md").read_text(encoding="utf-8")
    require(all(i in readme for i in IDS), "the README names every recipe")

    print("STATIC CONTRACT PASS: recipes 3 to 6 match the hand counts worked out from assembly.json.")
    print("RECIPES W23-AS-03 to W23-AS-06: NOT RUN (static only).")


if __name__ == "__main__":
    main()
