# Ancilliary pieces

## Container for cross-compiling

This directory contains the custom docker image configuration file used for cross-compiling for a Raspberry Pi (armv7), as well as the supporting files copied from the [cross](https://crates.io/crates/cross) project.

The command (run from this directory) used to build the container image is:

    podman build -t my/gtk-armv7-unknown-linux-gnueabihf . -f ./Dockerfile.gtk-armv7-unknown-linux-gnueabihf

See the main README for running the container to cross-compile.