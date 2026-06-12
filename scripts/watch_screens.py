#!/usr/bin/env python3
"""Pop a VNC viewer for each vmlab guest screen.

Scans the vmlab runtime dir for per-VM vnc.sock files and launches a
detached viewer for each new one. The viewer command comes from
$VMLAB_VIEWER (default: gvncviewer); it receives the socket path as its
only argument. Viewers outlive this script.

Modes:
  watch_screens.py                  watch until Ctrl-C
  watch_screens.py --once           open viewers for current sockets, exit
  watch_screens.py -- CMD ARG...    watch while CMD runs (vmlab up / builds)
"""

import argparse
import os
import signal
import socket
import subprocess
import sys
import time
from pathlib import Path


def runtime_root() -> Path:
    xdg = os.environ.get("XDG_RUNTIME_DIR")
    base = Path(xdg) if xdg else Path(f"/tmp/vmlab-{os.getuid()}")
    return base / "vmlab" / "labs"


def alive(sock: Path) -> bool:
    """vnc.sock files outlive `vmlab down` — only count connectable ones."""
    s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    s.settimeout(0.2)
    try:
        s.connect(str(sock))
        return True
    except OSError:
        return False
    finally:
        s.close()


def sockets(root: Path, lab: str | None) -> set[Path]:
    pattern = f"{lab}/vms/*/vnc.sock" if lab else "*/vms/*/vnc.sock"
    return {p for p in root.glob(pattern) if p.is_socket() and alive(p)}


def open_viewer(sock: Path) -> None:
    viewer = os.environ.get("VMLAB_VIEWER", "gvncviewer")
    lab, vm = sock.parts[-4], sock.parts[-2]
    print(f"screen: {lab}/{vm}", flush=True)
    subprocess.Popen(
        [viewer, str(sock)],
        start_new_session=True,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--lab", help="only this lab's screens (default: all)")
    ap.add_argument("--once", action="store_true", help="open and exit")
    ap.add_argument("cmd", nargs="*", help="command to run while watching")
    args = ap.parse_args()

    root = runtime_root()
    seen: set[Path] = set()
    child = subprocess.Popen(args.cmd) if args.cmd else None
    try:
        while True:
            for sock in sorted(sockets(root, args.lab) - seen):
                seen.add(sock)
                open_viewer(sock)
            if args.once:
                if not seen:
                    print("no guest screens found — is the lab up?", file=sys.stderr)
                    return 1
                return 0
            if child is not None and child.poll() is not None:
                return child.returncode
            time.sleep(0.5)
    except KeyboardInterrupt:
        if child is not None:
            child.send_signal(signal.SIGINT)
            return child.wait()
        return 0


if __name__ == "__main__":
    sys.exit(main())
