import { For, Show, createResource } from "solid-js";
import {
  state,
  vmStart,
  vmStop,
  vmRestart,
  takeSnapshot,
  restoreSnapshot,
  deleteSnapshot,
  look,
  osOf,
  archOf,
  fmtMem,
} from "../store";
import { vmSnapshots } from "../api";
import * as I from "./icons";
import VncScreen from "./VncScreen";

export default function MachineView() {
  // All of these are accessors so the view tracks the selected VM reactively —
  // switching machines re-runs them rather than pinning to the first one.
  const vm = () => state.status?.vms.find((v) => v.name === state.view.vm);
  const on = () => vm()?.state === "running";
  const lk = () => {
    const v = vm();
    return v ? look(v) : { label: "", dot: "var(--fg-3)", cls: "sb-stop" };
  };
  const segments = () =>
    vm()
      ?.nics.map((n) => n.segment)
      .filter(Boolean)
      .join(", ") || "—";

  // Re-fetched whenever the selected VM (or its power state) changes, and
  // explicitly after taking a new snapshot.
  const [snaps, { refetch }] = createResource(
    () => (state.view.vm ? `${state.currentLab}/${state.view.vm}/${vm()?.state}` : false),
    () => vmSnapshots(state.currentLab!, state.view.vm!).catch(() => []),
  );

  const takeVmSnapshot = async () => {
    const name = prompt(`Snapshot name for ${vm()!.name}:`);
    if (!name) return;
    await takeSnapshot(name, vm()!.name);
    refetch();
  };

  const delVmSnapshot = async (name: string) => {
    if (!confirm(`Delete snapshot "${name}" of ${vm()!.name}?`)) return;
    await deleteSnapshot(vm()!.name, name);
    refetch();
  };

  return (
    <Show
      when={vm()}
      fallback={
        <div class="body">
          <div class="csub">Machine not found.</div>
        </div>
      }
    >
      <header class="chead">
        <div>
          <div class="eyebrow">// machine</div>
          <h1 class="ctitle">
            {vm()!.name}
            <span class={`statebadge ${lk().cls}`}>
              <span class="sdot" style={`background:${lk().dot}`} />
              {lk().label}
            </span>
          </h1>
          <div class="csub">
            {osOf(vm()!)} · {archOf(vm()!)} · {vm()!.template}
          </div>
        </div>
        <div class="actions">
          <Show when={!on()}>
            <button class="btn btn-primary" onClick={() => vmStart(vm()!.name)}>
              <I.Power />
              Power on
            </button>
          </Show>
          <Show when={on()}>
            <button class="btn" onClick={() => vmStop(vm()!.name)}>
              <I.Power />
              Power off
            </button>
          </Show>
          <button class="btn" classList={{ dis: !on() }} onClick={() => vmRestart(vm()!.name)}>
            <I.Restart />
            Restart
          </button>
        </div>
      </header>

      <div class="body">
        <div class="vmlayout">
          <div class="screencol">
            <VncScreen lab={state.currentLab!} vm={vm()!.name} powered={on()} />
          </div>
          <div class="vmside">
            <div class="card">
              <div class="cardhd">Machine</div>
              <div class="cardbd">
                <KV k="Template" v={vm()!.template} />
                <KV k="vCPU" v={vm()!.cpus ? String(vm()!.cpus) : "default"} />
                <KV k="Memory" v={vm()!.memory ? fmtMem(vm()!.memory) : "default"} />
                <KV k="Arch" v={archOf(vm()!)} />
                <KV k="Address" v={vm()!.ip ?? "—"} />
                <KV k="MAC" v={vm()!.nics[0]?.mac ?? "—"} />
                <KV k="Segment" v={segments()} />
              </div>
            </div>
            <div class="card">
              <div class="cardhd">Snapshots</div>
              <div class="cardbd">
                <Show
                  when={(snaps()?.length ?? 0) > 0}
                  fallback={
                    <div class="csub" style="padding:8px 0">
                      No snapshots yet.
                    </div>
                  }
                >
                  <For each={snaps()}>
                    {(sn) => (
                      <div class="snaprow">
                        <span class="snapic">
                          <I.Camera />
                        </span>
                        <div class="snapmeta">
                          <div class="snapname">{sn.name}</div>
                          <div class="snaptime">{sn.online ? "online" : "offline"}</div>
                        </div>
                        <button
                          class="snaprestore"
                          onClick={() => restoreSnapshot(sn.name, vm()!.name)}
                        >
                          Restore
                        </button>
                        <button
                          class="snaprestore"
                          style="color:var(--danger-fg)"
                          onClick={() => delVmSnapshot(sn.name)}
                        >
                          Delete
                        </button>
                      </div>
                    )}
                  </For>
                </Show>
                <button
                  class="btn"
                  style="width:100%;margin-top:12px;height:32px"
                  onClick={takeVmSnapshot}
                >
                  <I.Camera />
                  Take snapshot
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </Show>
  );
}

function KV(p: { k: string; v: string }) {
  return (
    <div class="kv">
      <span class="kvk">{p.k}</span>
      <span class="kvv">{p.v}</span>
    </div>
  );
}
