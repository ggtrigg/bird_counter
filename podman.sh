#!/bin/sh
podman run -e "XARGO_HOME=/xargo" -e "CARGO_HOME=/cargo" -e "CARGO_TARGET_DIR=./target" -e "CROSS_RUNNER=" -v "${HOME}/.cargo:/cargo:Z" -v "${PWD}:/project:Z" -v "${HOME}/.rustup/toolchains/stable-x86_64-unknown-linux-gnu:/rust:Z,ro" -w /project -it --rm --name bird_counter localhost/my/gtk-armv7-unknown-linux-gnueabihf cargo build --release --target armv7-unknown-linux-gnueabihf
