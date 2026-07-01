// Provision for the Docker sample lab (./vmlab.wcl): make the /share bind
// mount usable inside the Alpine guest.
//
// The lab daemon auto-mounts every declared share over SMB, but a Linux guest
// needs two things a base Alpine image lacks:
//   - cifs-utils — the mount.cifs userspace helper (kernel CIFS alone cannot
//     satisfy `mount -t cifs`);
//   - the mount point — the daemon's mount step does not create it.
// We install the helper and create /mnt/share, then wait for the daemon's
// mount (retried every ~10s) to land. NAT egress on the "lan" segment lets
// apk reach the package mirror.

use vmlab

fn main(lab: Lab) {
    let vm = lab.vm("alpine").expect("no alpine vm in lab")
    vm.wait_ready(600).expect("alpine never became ready")

    let r = vm.exec_timeout("/sbin/apk", ["add", "--no-cache", "cifs-utils"], 300)
        .expect("running apk failed")
    if r.exit_code != 0 {
        lab.log("apk add cifs-utils failed:\n" + r.stderr)
        return
    }

    vm.exec("/bin/mkdir", ["-p", "/mnt/share"]).expect("mkdir /mnt/share failed")
    lab.log("alpine: cifs-utils installed, /mnt/share created — waiting for the daemon to mount the share")

    for i in 0..30 {
        match vm.exec("/bin/sh", ["-c", "mountpoint -q /mnt/share"]) {
            Ok(m) => {
                if m.exit_code == 0 {
                    lab.log("alpine: /share is mounted at /mnt/share — files in ./docker/share are now visible in the guest")
                    return
                }
            }
            Err(e) => lab.log("mount check failed: " + e),
        }
        if i % 6 == 5 {
            lab.log(fmt("alpine: still waiting for /mnt/share ({}s)", (i + 1) * 5))
        }
        vmlab::sleep_ms(5000)
    }
    lab.log("alpine: /mnt/share did not mount within ~150s — check `vmlab logs`")
}
