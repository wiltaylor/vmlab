import { For, Show } from "solid-js";
import { state, showVm, look, osOf } from "../store";
import type { Vm } from "../api";

interface Row {
  vm: Vm;
  mac: string | null;
  ip: string | null;
}

export default function NetworkView() {
  const s = () => state.status;

  const rowsFor = (segName: string): Row[] => {
    const rows: Row[] = [];
    for (const vm of s()!.vms) {
      const nic = vm.nics.find((n) => n.segment === segName);
      if (nic) rows.push({ vm, mac: nic.mac, ip: nic.static_ip ?? vm.ip });
    }
    return rows;
  };

  return (
    <Show
      when={s()}
      fallback={
        <div class="body">
          <div class="csub">No lab selected.</div>
        </div>
      }
    >
      <header class="chead">
        <div>
          <div class="eyebrow">// network</div>
          <h1 class="ctitle">network</h1>
          <div class="csub">{s()!.segments.length} segments</div>
        </div>
      </header>
      <div class="body">
        <For each={s()!.segments}>
          {(seg) => {
            const rows = rowsFor(seg.name);
            return (
              <div class="segblock">
                <div class="seghd">
                  <div class="seghdl">
                    <span class="segtitle">{seg.name}</span>
                    <span class="segcidr2">{seg.subnet}</span>
                    <span class="seggw">gateway {seg.gateway}</span>
                  </div>
                  <div class="seghdr">
                    {seg.dhcp ? (
                      <span class="flag fl-on">dhcp</span>
                    ) : (
                      <span class="flag fl-off">static</span>
                    )}
                    {seg.nat ? <span class="flag fl-on">nat</span> : null}
                    <span class="seghosts">{rows.length} hosts</span>
                  </div>
                </div>
                <div class="tablecard">
                  <table class="tbl">
                    <thead>
                      <tr>
                        <th>Machine</th>
                        <th>OS</th>
                        <th>IP address</th>
                        <th>MAC</th>
                        <th class="tr">State</th>
                      </tr>
                    </thead>
                    <tbody>
                      <Show
                        when={rows.length}
                        fallback={
                          <tr>
                            <td colspan="5" class="mutedcell">
                              No machines on this segment.
                            </td>
                          </tr>
                        }
                      >
                        <For each={rows}>
                          {(r) => {
                            const lk = look(r.vm);
                            return (
                              <tr class="rowlink" onClick={() => showVm(r.vm.name)}>
                                <td>
                                  <span class="cellname">{r.vm.name}</span>
                                </td>
                                <td class="mutedcell">{osOf(r.vm)}</td>
                                <td class="ipcell">{r.ip ?? "—"}</td>
                                <td class="mutedcell">{r.mac ?? "—"}</td>
                                <td class="tr">
                                  <span class="statecell">
                                    <span class="sdot" style={`background:${lk.dot}`} />
                                    <span style={`color:${lk.dot}`}>{lk.label}</span>
                                  </span>
                                </td>
                              </tr>
                            );
                          }}
                        </For>
                      </Show>
                    </tbody>
                  </table>
                </div>
              </div>
            );
          }}
        </For>
      </div>
    </Show>
  );
}
