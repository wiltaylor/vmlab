import { Show } from "solid-js";
import { toast } from "../store";

export default function Toast() {
  return (
    <Show when={toast()}>
      <div class="toast">
        <svg
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <polyline points="20 6 9 17 4 12" />
        </svg>
        {toast()}
      </div>
    </Show>
  );
}
