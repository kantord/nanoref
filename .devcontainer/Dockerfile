# Rust (stable) on the standard devcontainer base (non-root `vscode` user,
# uid 1000 -- required for enwiro's containerized-launch path, which maps the
# host uid onto this image's own passwd entry via `--userns=keep-id`).
FROM mcr.microsoft.com/devcontainers/rust:1-bookworm

# Everything .pre-commit-config.yaml needs to run on commit: prek executes the
# hooks, and the `language: system` hooks call these standalone binaries.
# Installed from pinned release tarballs (no compiling); layouts differ per
# project, so extract into a scratch dir and locate the binary by name.
ARG PREK_VERSION=0.4.8
ARG TYPOS_VERSION=1.48.0
ARG CARGO_DENY_VERSION=0.20.2
ARG CARGO_MACHETE_VERSION=0.9.2
RUN set -eux; \
    fetch() { \
      bin="$1"; url="$2"; dir="/tmp/fetch-$bin"; mkdir -p "$dir"; \
      curl -fsSL "$url" | tar -xz -C "$dir"; \
      install -m 0755 "$(find "$dir" -type f -name "$bin" | head -n 1)" "/usr/local/bin/$bin"; \
      rm -rf "$dir"; \
    }; \
    fetch prek "https://github.com/j178/prek/releases/download/v${PREK_VERSION}/prek-x86_64-unknown-linux-musl.tar.gz"; \
    fetch typos "https://github.com/crate-ci/typos/releases/download/v${TYPOS_VERSION}/typos-v${TYPOS_VERSION}-x86_64-unknown-linux-musl.tar.gz"; \
    fetch cargo-deny "https://github.com/EmbarkStudios/cargo-deny/releases/download/${CARGO_DENY_VERSION}/cargo-deny-${CARGO_DENY_VERSION}-x86_64-unknown-linux-musl.tar.gz"; \
    fetch cargo-machete "https://github.com/bnjbvr/cargo-machete/releases/download/v${CARGO_MACHETE_VERSION}/cargo-machete-v${CARGO_MACHETE_VERSION}-x86_64-unknown-linux-musl.tar.gz"
