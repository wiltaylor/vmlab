import {
  For,
  Show,
  createEffect,
  createMemo,
  createSignal,
  onCleanup,
} from "solid-js";
import { state } from "../store";
import { logsWsUrl } from "../api";
import type { LogEntry } from "../api";

// Cap the in-memory buffer; oldest lines drop off the top.
const MAX = 5000;

// A log line plus its parsed epoch-ms time, so the view can sort chronologically
// without re-parsing the timestamp on every comparison.
type Row = LogEntry & { _t: number };

export default function LogsView() {
  const [entries, setEntries] = createSignal<Row[]>([]);
  const [vm, setVm] = createSignal("all");
  const [query, setQuery] = createSignal("");
  const [follow, setFollow] = createSignal(true);
  const [connected, setConnected] = createSignal(false);
  let pane: HTMLDivElement | undefined;

  // Distinct sources for the filter dropdown: lab + each VM in the lab.
  const sources = () => ["lab", ...(state.status?.vms ?? []).map((v) => v.name)];

  const filtered = createMemo(() => {
    const v = vm();
    const q = query().toLowerCase();
    return entries()
      .filter(
        (e) =>
          (v === "all" || e.source === v) &&
          (q === "" || e.text.toLowerCase().includes(q)),
      )
      .sort((a, b) => a._t - b._t); // chronological across all sources
  });

  // Auto-scroll to the bottom as lines arrive, unless the user scrolled up.
  createEffect(() => {
    filtered();
    if (follow() && pane) queueMicrotask(() => (pane!.scrollTop = pane!.scrollHeight));
  });

  const onScroll = () => {
    if (!pane) return;
    const atBottom = pane.scrollHeight - pane.scrollTop - pane.clientHeight < 40;
    setFollow(atBottom);
  };

  // (Re)connect the log stream whenever the current lab changes.
  createEffect(() => {
    const lab = state.currentLab;
    if (!lab) return;
    setEntries([]);
    let ws: WebSocket | null = null;
    let closed = false;

    const connect = () => {
      ws = new WebSocket(logsWsUrl(lab));
      ws.onopen = () => setConnected(true);
      ws.onclose = () => {
        setConnected(false);
        if (!closed) setTimeout(connect, 2000);
      };
      ws.onmessage = (msg) => {
        try {
          const e: LogEntry = JSON.parse(msg.data);
          const row: Row = { ...e, _t: e.ts ? Date.parse(e.ts) : Date.now() };
          setEntries((prev) => {
            const next = prev.length >= MAX ? prev.slice(prev.length - MAX + 1) : prev.slice();
            next.push(row);
            return next;
          });
        } catch {
          /* ignore malformed */
        }
      };
    };
    connect();

    onCleanup(() => {
      closed = true;
      ws?.close();
    });
  });

  const jumpToBottom = () => {
    setFollow(true);
    if (pane) pane.scrollTop = pane.scrollHeight;
  };

  return (
    <Show
      when={state.currentLab}
      fallback={
        <div class="body">
          <div class="csub">No lab selected.</div>
        </div>
      }
    >
      <header class="chead">
        <div>
          <div class="eyebrow">// logs</div>
          <h1 class="ctitle">logs</h1>
          <div class="csub">
            {filtered().length} lines
            {query() || vm() !== "all" ? ` (of ${entries().length})` : ""}
          </div>
        </div>
        <div class="logctl">
          <select class="logsel" value={vm()} onChange={(e) => setVm(e.currentTarget.value)}>
            <option value="all">all sources</option>
            <For each={sources()}>{(s) => <option value={s}>{s}</option>}</For>
          </select>
          <input
            class="logq"
            type="search"
            placeholder="filter…"
            value={query()}
            onInput={(e) => setQuery(e.currentTarget.value)}
          />
          <button class="logbtn" onClick={() => setEntries([])} title="Clear the view">
            clear
          </button>
          <button
            class="logbtn"
            classList={{ on: follow() }}
            onClick={jumpToBottom}
            title="Follow the tail"
          >
            follow
          </button>
          <span class={connected() ? "c-ok" : "c-dim"}>
            ● {connected() ? "live" : "offline"}
          </span>
        </div>
      </header>
      <div class="body">
        <div class="logpane" ref={pane} onScroll={onScroll}>
          <Show
            when={filtered().length}
            fallback={<div class="logempty">No log lines{query() ? " match the filter" : " yet"}.</div>}
          >
            <For each={filtered()}>
              {(e) => (
                <div class="logrow">
                  <span class="logts">{fmtTs(e.ts)}</span>
                  <span class="logsrc" classList={{ "logsrc-lab": e.source === "lab" }}>
                    {e.source}
                  </span>
                  <span class={`logstream ls-${e.stream}`}>{e.stream}</span>
                  <span class="logmsg">{e.text}</span>
                </div>
              )}
            </For>
          </Show>
        </div>
      </div>
    </Show>
  );
}

function fmtTs(ts?: string | null): string {
  if (!ts) return "";
  const d = new Date(ts);
  if (isNaN(d.getTime())) return "";
  return d.toLocaleTimeString(undefined, { hour12: false });
}
