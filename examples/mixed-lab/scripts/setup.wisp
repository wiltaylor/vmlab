// Provision for mixed-lab (PRD §10): wait for both guests, stand up nginx
// on the Ubuntu box (reachable from the host via the segment's 18080
// forward), and prove the SMB share landed on the Windows side.

use vmlab

fn setup(lab: Lab) -> Result[unit, string] {
    let win = lab.vm("winsrv")?
    let nix = lab.vm("nix01")?

    win.wait_ready(900)?
    nix.wait_ready(900)?
    lab.log("winsrv at " + win.ip()? + ", nix01 at " + nix.ip()?)

    let cmd = "apt-get update && apt-get install -y nginx && echo '<h1>mixed-lab: hello from nix01</h1>' > /var/www/html/index.html"
    let r = nix.exec_timeout("/bin/sh", ["-c", cmd], 600)?
    if r.exit_code != 0 {
        return Err("nginx install failed: " + r.stderr)
    }
    lab.log("nginx serving on nix01 — try: curl http://localhost:18080")

    // The daemon mounts shares as soon as the agent responds — S: is
    // normally there within seconds; the window is just safety margin.
    for i in 0..60 {
        match win.exec("cmd.exe", ["/c", "dir S:"]) {
            Ok(s) => {
                if s.exit_code == 0 {
                    lab.log("winsrv sees the share:\n" + s.stdout)
                    return Ok(())
                }
            }
            Err(e) => lab.log("share check failed: " + e),
        }
        if i % 6 == 5 {
            lab.log(fmt("still waiting for S: on winsrv ({}s)", (i + 1) * 5))
        }
        vmlab::sleep_ms(5000)
    }
    lab.log("S: never appeared on winsrv — check `vmlab logs`")
    Ok(())
}

fn main(lab: Lab) {
    setup(lab).expect("mixed-lab setup failed")
}
