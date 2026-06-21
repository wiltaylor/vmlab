# Vm

_wscript API object_

A VM handle: lifecycle, state, snapshots, keyboard/mouse, screen matching/OCR, and the guest agent.

## Lifecycle & state

| Method | Returns | Notes |
| --- | --- | --- |
| `vm.name()` | `string` |  |
| `vm.start()` / `vm.stop()` / `vm.stop_force()` / `vm.restart()` | `Result[unit, string]` | stop = graceful ladder (agent → ACPI → kill) |
| `vm.state()` | `string` | one of `"stopped"` / `"starting"` / `"running"` / `"stopping"` |
| `vm.is_ready()` | `bool` | Guest agent responding |
| `vm.wait_ready(timeout_secs: int)` | `Result[unit, string]` | Block until agent responds |
| `vm.wait_shutdown(timeout_secs: int)` | `Result[unit, string]` | Block until powered off |
| `vm.ip()` | `Result[string, string]` | Primary NIC IPv4 (DHCP lease / agent) |
| `vm.ip_nic(nic: int)` | `Result[string, string]` | By NIC index (0-based) |

## Snapshots

| Method | Returns |
| --- | --- |
| `vm.snapshot(name: string)` | `Result[unit, string]` — online or offline per current state |
| `vm.restore(name: string)` | `Result[unit, string]` — resumes running iff taken online |
| `vm.snapshots()` | `Result[List[string], string]` |
| `vm.delete_snapshot(name: string)` | `Result[unit, string]` |

## Keyboard & mouse

| Method | Returns | Notes |
| --- | --- | --- |
| `vm.send_keys(chord: string)` | `Result[unit, string]` | e.g. `"ctrl-alt-del"`, `"enter"`, `"shift-f5"` |
| `vm.type_text(text: string)` | `Result[unit, string]` | ~35 ms/key; `\n`→enter, `\t`→tab; **US ASCII only** |
| `vm.type_text_paced(text: string, delay_ms: int)` | `Result[unit, string]` | Custom inter-key delay |
| `vm.mouse_move(x: int, y: int)` | `Result[unit, string]` | Absolute, scaled to current screen size |
| `vm.mouse_click(button: string)` | `Result[unit, string]` | `"left"` / `"right"` / `"middle"` |
| `vm.mouse_drag(x1, y1, x2, y2: int)` | `Result[unit, string]` | Human-ish 8-step drag |

See [the chord-key reference](../references/fact_key_chords.md) for the full key-name vocabulary.

## Screen, image matching, OCR

| Method | Returns | Notes |
| --- | --- | --- |
| `vm.screenshot(path: string)` | `Result[string, string]` | Returns saved path. `""` → auto-named in `.vmlab/screenshots/` |
| `vm.wait_for_image(image: string, timeout_secs: int)` | `Result[Match, string]` | Threshold 0.9, polls every 1 s |
| `vm.wait_for_image_opts(image, timeout_secs, threshold: float, region: List[int])` | `Result[Match, string]` | `region` = `[x, y, w, h]` or `[]` |
| `vm.wait_for_any(images: List[string], timeout_secs: int)` | `Result[Match, string]` | First image to appear wins |
| `vm.find_image(image: string)` | `Result[Option[Match], string]` | Single shot, no waiting |
| `vm.ocr()` | `Result[string, string]` | Tesseract over the whole screen |
| `vm.ocr_region(region: List[int])` | `Result[string, string]` | Cropped `[x, y, w, h]` |
| `vm.wait_for_text(pattern: string, timeout_secs: int)` | `Result[Match, string]` | Regex over OCR text; only `m.text`/`m.score` meaningful (coords are 0) |

Reference images resolve relative to the **lab root**; convention is an `images/` folder beside `vmlab.wcl`. Matching is normalized cross-correlation on grayscale.

## Guest agent

| Method | Returns | Notes |
| --- | --- | --- |
| `vm.exec(cmd: string, args: List[string])` | `Result[ExecResult, string]` | 120 s timeout |
| `vm.exec_timeout(cmd, args, timeout_secs: int)` | `Result[ExecResult, string]` | Custom timeout |
| `vm.copy_to(local: string, guest_path: string)` | `Result[unit, string]` | local relative to lab root; guest path absolute |
| `vm.copy_from(guest_path: string, local: string)` | `Result[unit, string]` | Parent dirs created on host |

## Related

- [Lab](../references/entity_lab_api.md)

- [Match / ExecResult / Event](../references/entity_match_type.md)

- [Keyboard chord names](../references/fact_key_chords.md)

[← All entities](../references/entities_ref.md)
