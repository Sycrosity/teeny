`Spotify Mini Controller`
==================
![CI](https://github.com/Sycrosity/spotify-mini/actions/workflows/ci.yml/badge.svg)
<!-- [![Pages](https://github.com/cosmiccrew/galaxy/actions/workflows/pages.yml/badge.svg)](https://cosmiccrew.github.io/galaxy)
![Release](https://github.com/cosmiccrew/galaxy/actions/workflows/release.yml/badge.svg) -->

A teeny tiny Spotify Controller for esp32 devices.
-------

## Download & run

#### From source

1. Install rust at [rustup.rs](https://rustup.rs)
2. Install espup at [esp-rs/espup](https://github.com/esp-rs/espup)
3. Clone the repo `git clone https://github.com/Sycrosity/spotify-mini.git`
4. `cd spotify-mini`
5. `cargo run --release` (do not use `cargo run`! On embedded devices the runtime performance of debug mode is orders of magnitude slower.

-------

## Contributing

Any and all contributions are welcome! Pull requests are checked for `cargo test`, `cargo clippy` and `cargo +nightly fmt`. Note this project uses unstable cargo fmt settings, and requires installing and running cargo fmt on the nightly edition.

Before submitting a PR or issue, please run the following commands and follow their instructions:
1. `cargo clippy`
2. `cargo fmt`

#### Dev builds

TODO

-------

## Credits

TODO

-------

## License
Licensed under either of

 - Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
 - MIT license
   ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
