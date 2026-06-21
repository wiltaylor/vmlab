# Official vmlab runtime image (PRD §14): the vmlab binary plus its full
# runtime dependency set. Because the network fabric is userspace, the
# container needs NO --privileged, no extra capabilities, and no host network
# mode — `--device /dev/kvm` is the only host grant needed for acceleration
# (without it, vmlab falls back to TCG with a loud warning).
#
# vmlab depends on the sibling WCL and wscript workspaces via path deps, so the
# build context is the PARENT directory containing vmlab/, WCL/, and wscript/.
#
# Build:  docker build -t vmlab -f vmlab/Containerfile .      (run from ../)
#    or:  just image                                          (from vmlab/)
# Run:    docker run --rm -it --device /dev/kvm \
#           -v ~/.local/share/vmlab/templates:/root/.local/share/vmlab/templates \
#           -v "$PWD":/lab -w /lab vmlab vmlab up

# ---- builder ----------------------------------------------------------------
FROM rust:1.92-bookworm AS builder
WORKDIR /build
# Bring in vmlab plus its sibling path dependencies.
COPY vmlab ./vmlab
COPY WCL ./WCL
COPY wscript ./wscript
RUN cargo build --release --locked --manifest-path vmlab/Cargo.toml

# ---- runtime ----------------------------------------------------------------
FROM debian:bookworm-slim
# QEMU system emulators, firmware, swtpm, OCR, NAT, ISO/floppy tooling, SMB
# server, and a VNC-capable toolchain (PRD §14).
RUN apt-get update && apt-get install -y --no-install-recommends \
        qemu-system-x86 \
        qemu-system-arm \
        qemu-utils \
        ovmf \
        seabios \
        swtpm \
        tesseract-ocr \
        passt \
        xorriso \
        mtools \
        dosfstools \
        samba \
        ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/vmlab/target/release/vmlab /usr/local/bin/vmlab

# Documented volume mounts (PRD §14):
#   /root/.local/share/vmlab/templates  — the template store
#   /lab                                — the lab directory
# Everything else is container-ephemeral by design.
VOLUME ["/root/.local/share/vmlab/templates"]
WORKDIR /lab

# Entrypoint defaults to the supervisor in the foreground; lab daemons are its
# children. `docker exec` (or a second container sharing the socket volume)
# drives the CLI. A one-shot CI mode is `docker run vmlab vmlab up && ...`.
ENTRYPOINT ["vmlab"]
CMD ["daemon", "start"]
