import json, os, pathlib, socket, struct, time
report=pathlib.Path(os.environ["REPORT"])
port=int(os.environ["CONTROL_PORT"])
token=os.environ["SOLIDCRAFT_CONTROL_TOKEN"]
def call(method, params=None):
    with socket.create_connection(("127.0.0.1",port),timeout=180) as s:
        s.settimeout(180)
        s.sendall((json.dumps(dict(id=1,method=method,params=params or {},token=token))+"\n").encode())
        r=json.loads(s.makefile("rb").readline())
    with (report/"control.jsonl").open("a") as out:
        out.write(json.dumps(dict(method=method,params=params or {},reply=r))+"\n")
    if not r.get("ok"): raise RuntimeError(f"{method}: {r.get('error')}")
    return r.get("result")
def engine(cmd,params=None): return call("engine.execute",dict(command=cmd,params=params or {}))
def shot(name, size):
    p=report/(name+".png")
    call("ui.screenshot",dict(path=str(p)))
    with p.open("rb") as f: header=f.read(24)
    assert header[:8]==b"\x89PNG\r\n\x1a\n" and struct.unpack(">II",header[16:24])==size, (name,header)

window_size=(1280,720)
def handles():
    return call("ui.handles",dict(prefix=""))
def find_button(label):
    return [h for h in handles() if h["name"].startswith("dialog:button:") and h["name"].endswith(":"+label)]
def find_exact(name):
    return [h for h in handles() if h["name"]==name]
def wait_for(find, description, present=True):
    for _ in range(40):
        hits=find()
        if (present and len(hits)==1) or (not present and not hits):
            return hits[0] if present else None
        time.sleep(.25)
    raise AssertionError(f"{description}: expected {'one' if present else 'no'} visible handle; found {hits}")
def click_handle(h):
    x,y=h["x"],h["y"]
    assert 0<=x<window_size[0] and 0<=y<window_size[1], ("Control clipped",h,window_size)
    print("CLICK",h,flush=True)
    call("ui.click",dict(x=x,y=y))
    time.sleep(.45)
def button(label):
    return wait_for(lambda:find_button(label),label)
def click_button(label):
    click_handle(button(label))
def absent(label):
    wait_for(lambda:find_button(label),label,False)
def badge(count):
    label="No dimensions need attention" if count==0 else f"Dimensions needing attention: {count}"
    h=button(label)
    assert window_size[0]*.25<h["x"]<window_size[0]*.75 and 0<h["y"]<window_size[1]*.5, ("Status not in upper central area",h)
    return h
def repair_handles():
    return [h for h in handles() if h["name"].startswith("dialog:button:") and ":Repair Dimension " in h["name"]]
def repair_button():
    return wait_for(repair_handles,"Readable repair button")
def click_repair():
    click_handle(repair_button())
def inspect():
    return engine("drawing.dimension.inspect",dict(dimension=dim))
def candidate_name(pick):
    info=pick["edge"]
    return f"Edge {info['index']+1} of {info['body']}"
def choose_candidate(pick,eligible):
    click_handle(wait_for(lambda:find_exact("t240:candidate_combo"),"Candidate dropdown"))
    names={candidate_name(p) for p in eligible}
    assert len(names)==len(eligible),"Candidate names are not unique"
    for name in names:
        wait_for(lambda n=name:find_exact("dialog:list:"+n),name)
    click_handle(wait_for(lambda:find_exact("dialog:list:"+candidate_name(pick)),candidate_name(pick)))
def lost_unchanged():
    state=inspect()
    assert state["status"]=="lost" and state["record"]==before["record"],state

# Preserve the first failing frame and available control handles. Never mask the original exception.
import sys
def record_failure(exc_type, exc, traceback):
    try:
        call("ui.screenshot",dict(path=str(report/"failure.png")))
        (report/"failure_handles.json").write_text(json.dumps(handles(),indent=2))
        (report/"failure_ui.json").write_text(json.dumps(call("ui.inspect"),indent=2))
    except Exception as diagnostic_error:
        print("Failure diagnostics unavailable:",str(diagnostic_error),flush=True)
    sys.__excepthook__(exc_type,exc,traceback)
sys.excepthook=record_failure

for _ in range(180):
    try:
        os.kill(int(os.environ["APP_PID"]),0)
    except ProcessLookupError:
        raise RuntimeError("Native application exited: "+(report/"app.txt").read_text(errors="replace"))
    try:
        if call("ui.inspect").get("viewport"): break
    except (OSError,RuntimeError): pass
    time.sleep(1)
else: raise RuntimeError("Native control channel never ready")
call("ui.agent_cursor",dict(show=False))
call("ui.home",dict(open=False))
call("ui.prefs",dict(language="en"))
call("ui.resize",dict(width=1280,height=720))
time.sleep(2)
recipe=json.load(open("examples/acceptance/drawing_m2.json"))["commands"]
assert recipe
for step in recipe: result=engine(step["command"],step["params"])
view=result["view"]
drawing=engine("drawing.inspect")["id"]
added=engine("drawing.dimension.add",dict(view=view,kind=dict(radial=dict(edge=[25,20,20]))))
dim=added["dimension"]
assert added["status"]=="ok"
call("ui.workspace",dict(workspace="drawing"))
time.sleep(2)

def engine_attention(stage):
    status=engine("drawing.status")
    states=status.get("dimensions",[])
    count=sum(1 for d in states if d.get("out_of_date") or d.get("status") not in ("ok","overridden"))
    payload=dict(status=status, dimension_inspect=inspect(), expected_count=count)
    (report/("engine_attention_"+stage+".json")).write_text(json.dumps(payload,indent=2,ensure_ascii=False))
    print("ENGINE STATUS",stage,json.dumps(payload,ensure_ascii=False),flush=True)
    return count
initial_count=engine_attention("intact")
assert initial_count<=1,("Unexpected initial attention count",initial_count)
badge(initial_count)
if initial_count:
    print("ATTENTION BEFORE EDIT: dimension.add returned ok, but drawing.status still requires attention; investigate status reasons",flush=True)
shot("01_intact_1280x720",window_size)
engine("drawing.close")
call("ui.workspace",dict(workspace="design"))
engine("solid.fillet",dict(body="Plate",radius=1,edges=[[25,20,20]]))
engine("drawing.open",dict(drawing=drawing))
call("ui.workspace",dict(workspace="drawing"))
time.sleep(2)
before=inspect()
assert before["status"]=="lost" and before["candidates"],before
lost_count=engine_attention("lost")
assert lost_count == 1, ("Expected exactly one affected dimension",lost_count)
badge(lost_count)
shot("02_lost_collapsed_1280x720",window_size)
click_handle(badge(1))
repair_button()
shot("03_lost_expanded_1280x720",window_size)
click_handle(badge(1))
wait_for(repair_handles,"Closed status card",False)
click_handle(badge(1))
repair_button()
engine("drawing.dimension.repair",dict(dimension=dim,slot="edge",pick=dict(edge=[25,20,19])))
expected=inspect()
assert expected["status"]=="ok" and expected["text"]=="Ø10.00",expected
engine("edit.undo")
lost_unchanged()
badge(1)
eligible=[pick for pick in before["candidates"] if set(pick)=={"edge"}]
assert eligible
click_repair()
click_button("Review repair")
button("Confirm repair")
lost_unchanged()
shot("04_review_1280x720",window_size)
click_button("Cancel repair")
absent("Confirm repair")
lost_unchanged()
def repair_at_size(tag):
    for i,pick in enumerate(eligible):
        click_repair()
        choose_candidate(pick,eligible)
        shot(f"{tag}_candidate_{i}",window_size)
        click_button("Review repair")
        button("Confirm repair")
        lost_unchanged()
        shot(f"{tag}_review_{i}",window_size)
        click_button("Confirm repair")
        after=inspect()
        shot(f"{tag}_result_{i}",window_size)
        if after["status"]=="ok" and after["text"]=="Ø10.00" and after["record"]["kind"]==expected["record"]["kind"]:
            badge(0)
            shot(f"{tag}_repaired",window_size)
            engine("edit.undo")
            lost_unchanged()
            badge(1)
            shot(f"{tag}_undo",window_size)
            return
        if after["record"]!=before["record"] or after["status"]!="lost":
            engine("edit.undo")
            lost_unchanged()
        else:
            lost_unchanged()
    raise AssertionError("No visible candidate repaired the exact expected reference")
repair_at_size("05_1280x720")
call("ui.resize",dict(width=1024,height=640))
time.sleep(2)
window_size=(1024,640)
badge(1)
shot("06_small_window",window_size)
click_repair()
click_button("Review repair")
button("Confirm repair")
lost_unchanged()
shot("07_small_review",window_size)
click_button("Cancel repair")
absent("Confirm repair")
lost_unchanged()
repair_at_size("08_1024x640")
print("PASS: native status count, toggle, cancel, exact repair and undo at both sizes",flush=True)
