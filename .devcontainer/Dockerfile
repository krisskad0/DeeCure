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
USER vscode
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
        | sh -s -- -y --profile minimal --default-toolchain none
ENV PATH=/home/vscode/.cargo/bin:${PATH}
