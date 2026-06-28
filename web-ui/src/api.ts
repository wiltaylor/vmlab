// Typed fetch + WebSocket helpers against the vmlab-web backend, with bearer
// token handling. The token (issued by /api/login) lives in localStorage and
// is sent as an Authorization header on REST calls and a ?token= query param
// on WebSocket upgrades (browsers can't set WS headers).

const TOKEN_KEY = "vmlab_token";

export function getToken(): string {
  return localStorage.getItem(TOKEN_KEY) ?? "";
}
export function setToken(t: string): void {
  localStorage.setItem(TOKEN_KEY, t);
}
export function clearToken(): void {
  localStorage.removeItem(TOKEN_KEY);
}

export class Unauthorized extends Error {}

async function req(path: string, opts: RequestInit = {}): Promise<any> {
  const headers: Record<string, string> = {
    ...((opts.headers as Record<string, string>) ?? {}),
  };
  const t = getToken();
  if (t) headers["Authorization"] = `Bearer ${t}`;
  if (opts.body && !headers["Content-Type"]) {
    headers["Content-Type"] = "application/json";
  }
  const res = await fetch(path, { ...opts, headers });
  if (res.status === 401) {
    clearToken();
    throw new Unauthorized("authentication required");
  }
  if (!res.ok) {
    let msg = res.statusText;
    try {
      msg = (await res.json()).error ?? msg;
    } catch {
      /* keep statusText */
    }
    throw new Error(msg);
  }
  const ct = res.headers.get("content-type") ?? "";
  return ct.includes("json") ? res.json() : res;
}

const post = (path: string, body?: unknown) =>
  req(path, { method: "POST", body: body ? JSON.stringify(body) : undefined });

// --- auth -----------------------------------------------------------------

export interface AuthProbe {
  auth_required: boolean;
  user: string | null;
}
export const authProbe = (): Promise<AuthProbe> => req("/api/auth");
export const login = (username: string, password: string): Promise<{ token: string }> =>
  post("/api/login", { username, password });
export const logout = (): Promise<unknown> => post("/api/logout");

// --- labs -----------------------------------------------------------------

export interface LabEntry {
  name: string;
  root?: string;
  state?: string;
}
export interface Nic {
  segment: string | null;
  mac: string | null;
  static_ip: string | null;
}
export interface Vm {
  name: string;
  state: string;
  ready: boolean;
  ip: string | null;
  template: string;
  arch: string | null;
  cpus: number | null;
  memory: number | null;
  nics: Nic[];
}
export interface Segment {
  name: string;
  subnet: string;
  gateway: string;
  nat: boolean;
  dhcp: boolean;
}
export interface LabStatus {
  lab: string;
  vms: Vm[];
  segments: Segment[];
}
export interface Snapshot {
  name: string;
  online: boolean;
  taken_at?: string;
}

export const listLabs = (): Promise<LabEntry[]> => req("/api/labs");
export const labStatus = (lab: string): Promise<LabStatus> =>
  req(`/api/labs/${encodeURIComponent(lab)}`);
export const labAction = (lab: string, action: "up" | "down" | "destroy") =>
  post(`/api/labs/${encodeURIComponent(lab)}/${action}`);
export const vmAction = (
  lab: string,
  vm: string,
  action: "start" | "stop" | "restart" | "destroy",
) => post(`/api/labs/${encodeURIComponent(lab)}/vms/${encodeURIComponent(vm)}/${action}`);
export const sendKeys = (lab: string, vm: string, keys: string) =>
  post(`/api/labs/${encodeURIComponent(lab)}/vms/${encodeURIComponent(vm)}/sendkeys`, {
    keys,
  });
export const vmSnapshots = (lab: string, vm: string): Promise<Snapshot[]> =>
  req(`/api/labs/${encodeURIComponent(lab)}/vms/${encodeURIComponent(vm)}/snapshots`);
export const deleteSnapshot = (lab: string, vm: string, name: string) =>
  req(
    `/api/labs/${encodeURIComponent(lab)}/vms/${encodeURIComponent(vm)}/snapshots/${encodeURIComponent(name)}`,
    { method: "DELETE" },
  );
export const takeSnapshot = (lab: string, name: string, vm?: string) =>
  post(`/api/labs/${encodeURIComponent(lab)}/snapshots`, { name, vm });
export const restoreSnapshot = (lab: string, name: string, vm?: string) =>
  post(
    `/api/labs/${encodeURIComponent(lab)}/snapshots/${encodeURIComponent(name)}/restore`,
    { vm },
  );

// --- websockets -----------------------------------------------------------

export function wsUrl(path: string): string {
  const proto = location.protocol === "https:" ? "wss" : "ws";
  const t = getToken();
  const q = t ? `?token=${encodeURIComponent(t)}` : "";
  return `${proto}://${location.host}${path}${q}`;
}

export interface DaemonEvent {
  event: string;
  lab: string;
  data: any;
  ts: string;
}
