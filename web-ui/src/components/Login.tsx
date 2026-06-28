import { Show, createSignal } from "solid-js";
import { doLogin, state } from "../store";

export default function Login() {
  const [user, setUser] = createSignal(state.authUser ?? "");
  const [pass, setPass] = createSignal("");
  const [err, setErr] = createSignal("");
  const [busy, setBusy] = createSignal(false);

  const submit = async (e: Event) => {
    e.preventDefault();
    setErr("");
    setBusy(true);
    try {
      await doLogin(user(), pass());
    } catch (ex) {
      setErr(String(ex instanceof Error ? ex.message : ex));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div class="mback" style="position:fixed">
      <form class="modal" style="width:380px" onSubmit={submit}>
        <div class="modalhd">
          <div class="modaltitle">
            <span class="bvm">vm</span>
            <span class="blab">lab</span> sign in
          </div>
          <div class="modalsub">Enter your credentials to access the console.</div>
        </div>
        <div class="modalbd">
          <label class="flbl">Username</label>
          <input
            class="input"
            value={user()}
            onInput={(e) => setUser(e.currentTarget.value)}
            autofocus
          />
          <label class="flbl" style="margin-top:14px">
            Password
          </label>
          <input
            class="input"
            type="password"
            value={pass()}
            onInput={(e) => setPass(e.currentTarget.value)}
          />
          <Show when={err()}>
            <div class="csub" style="color:var(--danger-fg);margin-top:10px">
              {err()}
            </div>
          </Show>
        </div>
        <div class="modalft">
          <button class="btn btn-primary" type="submit" classList={{ dis: busy() }}>
            {busy() ? "Signing in…" : "Sign in"}
          </button>
        </div>
      </form>
    </div>
  );
}
