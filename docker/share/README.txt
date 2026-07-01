Files placed in this directory appear inside any VM that declares a share
pointing at /share, e.g.:

    share { host = "/share" guest = "/mnt/share" }   # Linux guest
    share { host = "/share" guest = "S:" }           # Windows guest

This directory is bind-mounted to /share in the container (see compose.yaml).
The shipped sample lab (../lab/vmlab.wcl) mounts it into the "alpine" VM at
/mnt/share. See ../README.md ("Sharing files into your VMs") for the guest
prerequisites and the write-back ownership caveat.
