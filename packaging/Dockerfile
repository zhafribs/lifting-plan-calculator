# The portable AppImage build, for environments where Docker is preferred to
# bubblewrap. Build from the repository root:
#
#   docker build -f packaging/Dockerfile -t lpc-builder .
#   docker run --rm -v "$PWD":/work -w /work/src-tauri \
#     -e CARGO_TARGET_DIR=/work/src-tauri/target-ubuntu \
#     -e APPIMAGE_EXTRACT_AND_RUN=1 \
#     lpc-builder cargo tauri build --bundles appimage
#
# The AppImage lands in src-tauri/target-ubuntu/release/bundle/appimage/.
# Ubuntu 22.04 gives the same glibc 2.35 floor as the bubblewrap script.

FROM ubuntu:22.04

ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get install -y --no-install-recommends \
      build-essential curl wget git ca-certificates pkg-config file patchelf \
      python3 xz-utils \
      libwebkit2gtk-4.1-dev libgtk-3-dev librsvg2-dev libssl-dev \
      zlib1g-dev libsoup-3.0-dev \
  && rm -rf /var/lib/apt/lists/*

ENV RUSTUP_HOME=/opt/rustup \
    CARGO_HOME=/opt/cargo \
    PATH=/opt/cargo/bin:$PATH \
    APPIMAGE_EXTRACT_AND_RUN=1

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
      | sh -s -- -y --profile minimal --default-toolchain stable \
  && cargo install tauri-cli --locked

WORKDIR /work
