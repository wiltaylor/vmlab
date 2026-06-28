import { For, Show, createResource, createSignal, type JSX } from "solid-js";
import {
  state,
  startAll,
  stopAll,
  destroyLab,
  takeSnapshot,
  restoreSnapshot,
  deleteLabSnapshot,
  labSnapshotList,
  vmStart,
  vmStop,
  showVm,
  look,
  osOf,
  archOf,
  fmtMem,
} from "../store";
import * as I from "./icons";

function fmtTime(t: string): string {
  if (!t) return "";
  const d = new Date(t);
  return isNaN(d.getTime()) ? t : d.toLocaleString();
}

export default function LabView() {
  const s = () => state.status;
  const running = () => s()?.vms.filter((v) => v.state === "running").length ?? 0;
  const total = () => s()?.vms.length ?? 0;
  const vcpu = () => s()?.vms.reduce((a, v) => a + (v.cpus ?? 0), 0) ?? 0;
  const mem = () => s()?.vms.reduce((a, v) => a + (v.memory ?? 0), 0) ?? 0;

  const snapshotLab = () => {
    const name = prompt("Snapshot name for the whole lab:");
    if (name) takeSnapshot(name);
  };

  // Restore-lab picker: open a modal listing every snapshot across the lab.
  const [restoreOpen, setRestoreOpen] = createSignal(false);
  const [labSnaps, { refetch: refetchLabSnaps }] = createResource(restoreOpen, (open) =>
    open ? labSnapshotList() : Promise.resolve([]),
  );
  const pickRestore = (name: string) => {
    setRestoreOpen(false);
    restoreSnapshot(name);
  };
  const delLabSnapshot = async (name: string) => {
    if (!confirm(`Delete snapshot "${name}" from every VM in the lab?`)) return;
    await deleteLabSnapshot(name);
    refetchLabSnaps();
  };
  const destroy = () => {
    if (confirm("Destroy this lab? Clones and lab-local state are deleted.")) {
      destroyLab();
    }
  };

  return (
    <Show
      when={s()}
      fallback={
        <div class="body">
          <div class="csub">No lab selected, or the lab daemon isn't reachable.</div>
        </div>
      }
    >
      <header class="chead">
        <div>
          <div class="eyebrow">// lab</div>
          <h1 class="ctitle">{s()!.lab}</h1>
          <div class="csub">
            {total()} machines · {s()!.segments.length} segments
          </div>
        </div>
        <div class="actions">
          <button class="btn btn-primary" onClick={startAll}>
            <I.Play />
            Start all
          </button>
          <button class="btn" onClick={stopAll}>
            <I.Stop />
            Stop all
          </button>
          <button class="btn" onClick={snapshotLab}>
            <I.Camera />
            Snapshot lab
          </button>
          <button class="btn" onClick={() => setRestoreOpen(true)}>
            <I.Restore />
            Restore lab
          </button>
          <button class="btn btn-danger" onClick={destroy}>
            <I.Trash />
            Destroy lab
          </button>
        </div>
      </header>

      <div class="body">
        <div class="statgrid">
          <Stat icon={<I.Servers />} k="Machines up" v={String(running())} u={`/ ${total()}`} />
          <Stat icon={<I.Cpu />} k="Allocated vCPU" v={vcpu() ? String(vcpu()) : "—"} u="cores" />
          <Stat icon={<I.Memory />} k="Memory" v={fmtMem(mem() || null)} u="" />
          <Stat icon={<I.Nodes />} k="Segments" v={String(s()!.segments.length)} u="" />
        </div>

        <h3 class="sectitle">Machines</h3>
        <For each={s()!.vms}>
          {(vm) => {
            const lk = look(vm);
            const on = () => vm.state === "running";
            return (
              <div class="lvm">
                <span class="sdot" style={`background:${lk.dot}`} />
                <div style="min-width:0">
                  <div class="lvmname">{vm.name}</div>
                  <div class="lvmos">
                    {osOf(vm)} · {archOf(vm)}
                  </div>
                </div>
                <span class="lvmstate" style={`color:${lk.dot}`}>
                  {lk.label}
                </span>
                <span class="lvmip">{vm.ip ?? "—"}</span>
                <div class="miniact">
                  <button
                    class="mbtn"
                    onClick={() => (on() ? vmStop(vm.name) : vmStart(vm.name))}
                  >
                    {on() ? "Stop" : "Start"}
                  </button>
                  <button class="mbtn" onClick={() => showVm(vm.name)}>
                    <I.Monitor />
                    Console
                  </button>
                </div>
              </div>
            );
          }}
        </For>
      </div>

      <Show when={restoreOpen()}>
        <div class="mback" onClick={() => setRestoreOpen(false)}>
          <div class="modal" onClick={(e) => e.stopPropagation()}>
            <div class="modalhd">
              <div class="modaltitle">Restore lab</div>
              <div class="modalsub">
                Roll {s()!.lab} back to a saved snapshot. Choose one to restore.
              </div>
            </div>
            <div class="modalbd">
              <Show
                when={(labSnaps()?.length ?? 0) > 0}
                fallback={
                  <div class="csub" style="padding:6px 2px 12px">
                    {labSnaps.loading
                      ? "Loading snapshots…"
                      : "No snapshots found in this lab."}
                  </div>
                }
              >
                <div class="snaplist">
                  <For each={labSnaps()}>
                    {(snap) => (
                      <div class="snapopt">
                        <span class="snapic">
                          <I.Camera />
                        </span>
                        <div class="snapmeta">
                          <div class="snapname">{snap.name}</div>
                          <div class="snaptime">{fmtTime(snap.taken_at)}</div>
                        </div>
                        <button class="snaprestore" onClick={() => pickRestore(snap.name)}>
                          Restore
                        </button>
                        <button
                          class="snaprestore"
                          style="color:var(--danger-fg)"
                          onClick={() => delLabSnapshot(snap.name)}
                        >
                          Delete
                        </button>
                      </div>
                    )}
                  </For>
                </div>
              </Show>
            </div>
            <div class="modalft">
              <button class="btn" onClick={() => setRestoreOpen(false)}>
                Cancel
              </button>
            </div>
          </div>
        </div>
      </Show>
    </Show>
  );
}

function Stat(p: { icon: JSX.Element; k: string; v: string; u: string }) {
  return (
    <div class="stat">
      <div class="statk">
        {p.icon}
        {p.k}
      </div>
      <div class="statv">
        {p.v}
        {p.u ? <span class="statu">{p.u}</span> : null}
      </div>
    </div>
  );
}
