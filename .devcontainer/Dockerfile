# The DeeCure dev container: the same Ubuntu and packages as docs/04-development/getting-started.md.
# Used by GitHub Codespaces and VS Code "Reopen in Container" (getting-started.md, "Or: start in a dev container").
FROM mcr.microsoft.com/devcontainers/base:ubuntu-24.04@sha256:d94c97dd9cacf183d0a6fd12a8e87b526e9e928307674ae9c94139139c0c6eae

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        build-essential curl wget file git pkg-config \
        libwebkit2gtk-4.1-dev libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev \
        libsystemd-dev libsqlite3-dev socat \
        debhelper devscripts lintian desktop-file-utils appstream \
    && rm -rf /var/lib/apt/lists/*

# rustup only: the Rust version comes from rust-toolchain.toml when the container is created.
# A fixed rustup-init, checked against its published SHA-256 before it runs (never curl | sh).
# To update: the version is in https://static.rust-lang.org/rustup/release-stable.toml, and each
# sum in https://static.rust-lang.org/rustup/archive/<version>/<target>/rustup-init.sha256.
USER vscode
ARG RUSTUP_VERSION=1.29.3
RUN set -eu; \
    case "$(dpkg --print-architecture)" in \
        amd64) target=x86_64-unknown-linux-gnu; sum=b1c8cb14e30889e7e2999e6b69e62d9e9b9ffc0f4df8180564d306d9680e6c55 ;; \
        arm64) target=aarch64-unknown-linux-gnu; sum=9f608c142e0e38a78957d0661ba1dfaf0b6fb37184cfaa4b9a0aa57af6184282 ;; \
        *) echo "no rustup-init is pinned for $(dpkg --print-architecture)" >&2; exit 1 ;; \
    esac; \
    curl --proto '=https' --tlsv1.2 -sSfo /tmp/rustup-init \
        "https://static.rust-lang.org/rustup/archive/${RUSTUP_VERSION}/${target}/rustup-init"; \
    echo "${sum}  /tmp/rustup-init" | sha256sum -c -; \
    chmod +x /tmp/rustup-init; \
    /tmp/rustup-init -y --profile minimal --default-toolchain none; \
    rm /tmp/rustup-init
ENV PATH=/home/vscode/.cargo/bin:${PATH}
