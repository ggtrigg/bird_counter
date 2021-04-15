# Bird Counter

This is a simple app which displays one or more bird types (e.g. Sulphur Crested Cockatoo) and allows for the user to
indicate a "sighting" of that bird type for that day. Sightings are logged (in a sqlite3 database) and can be used to
report bird sightings per time period.

The intention is to run this on a Raspberry Pi with a touch screen as a simple and handy device for keeping track of
bird type occurances of the year.

![Screenshot](https://github.com/ggtrigg/bird_counter/blob/master/images/Screenshot%20from%202021-04-14%2013-22-25.png)

## Installation

Ultimately just the binary (bird_counter) and the image directory is required - once cross-compiling for the Raspberry Pi is working.

## Usage

The intention is to launch this automatically when the Raspberry Pi boots.

## Development

### Cross-compiling

To build the ARM version using the custom container (see misc/README.md) use the following command:

    podman run -e "XARGO_HOME=/xargo" -e "CARGO_HOME=/cargo" -e "CARGO_TARGET_DIR=/target" -e "CROSS_RUNNER=" -v "${HOME}/.cargo:/cargo:Z" -v "${HOME}/src/rust/bird_counter:/bird_counter:Z" -v "${HOME}/.rustup/toolchains/stable-x86_64-unknown-linux-gnu:/rust:Z,ro" -v "${HOME}/src/rust/bird_counter/target:/target:Z" -w /bird_counter -it localhost/my/gtk-armv7-unknown-linux-gnueabihf cargo build --target armv7-unknown-linux-gnueabihf

(If you're using docker you can replace `podman` with `docker` in that command.)

This command is contained in the `podman.sh` script for convenience.

## Contributing

1. Fork it (<https://github.com/ggtrigg/bird_counter/fork>)
2. Create your feature branch (`git checkout -b my-new-feature`)
3. Commit your changes (`git commit -am 'Add some feature'`)
4. Push to the branch (`git push origin my-new-feature`)
5. Create a new Pull Request

## Contributors

- [Glenn Trigg](https://github.com/ggtrigg) - creator and maintainer
