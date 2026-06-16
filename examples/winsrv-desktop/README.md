# winsrv-desktop — watch a Windows Server 2025 desktop

The smallest useful lab: one Windows Server 2025 VM on a NAT'd segment,
with its display surfaced on the host so you can drive the desktop by hand.
It exercises vmlab's console access (PRD §11) — `gui = true` to auto-open
a viewer, or `vmlab console` to attach one on demand.

Prerequisite — build the template first:

```sh
(cd ../templates/windows-server-2025 && ./fetch-deps.sh && vmlab template build)
```

Run it:

```sh
vmlab validate
vmlab up                # boots winsrv headless; with gui = true a viewer opens
vmlab status            # winsrv ready at 10.80.0.10
vmlab console winsrv    # reattach a viewer any time (closing it just disconnects)
vmlab down              # clone retained; `vmlab destroy` deletes it
```

Guest credentials: `Administrator` / `vmlab123!`.

## Showing the UI

Every VM always runs **headless**, serving a VNC display on a unix
socket. `gui = true` (set on the lab, inherited by every VM) just makes
`vmlab up` launch a VNC **viewer** against that socket per guest. Because
the viewer is a separate process, closing its window only disconnects —
the VM keeps running, and `vmlab console winsrv` reattaches. Drop `gui`
to `false` on a VM to skip the auto-opened viewer for that one.

vmlab finds a viewer automatically — a `viewer` in host config, else
`remote-viewer` / `gvncviewer` / `vncviewer` on `PATH`. `remote-viewer`
(virt-viewer) dials the VNC unix socket directly; `gvncviewer`/`vncviewer`
are TCP-only, so vmlab bridges the socket for them. Either way neither
`vmlab up` nor `vmlab console` blocks your terminal — a TCP viewer's
bridge runs in a detached helper that exits when you close the window.
On WSL2 (viewer on the Windows side) `vmlab console --tcp` bridges to a
localhost port to attach a Windows client.
