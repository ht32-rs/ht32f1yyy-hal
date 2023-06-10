ht32f1yyy-hal
=============

This crate implements a hardware abstraction layer for the Holtek HT32F1YYY chip family.

It relies on the [ht32f1yyy][] peripheral access crate to provide appropriate
register definitions, and implements a partial set of the [embedded-hal][] traits.

Much of the implementation was adapted from other HAL crates, like [ht32f5xxxx-hal][]
and those in the [stm32-rs organisation][stm32-rs].

Collaboration on this crate is highly welcome, as are pull requests!

Getting Started
---------------

The [examples folder](examples/) contains several example programs. To compile
them, specify the target device in a cargo feature:

```
$ cargo build --features=ht32f1755,rt
```

The examples make use of [defmt](https://github.com/knurling-rs/defmt)
for logging with deferred formatting.
To be able to flash the firmware and view the log messages,
you will need to use a tool like
[probe-rs-cli](https://github.com/probe-rs/probe-rs/tree/master/cli) or
[probe-run](https://github.com/knurling-rs/probe-run).

To use ht32f1yyy-hal as a dependency in a standalone project the
target device feature must be specified in the `Cargo.toml` file:

```toml
[dependencies]
cortex-m = "0.7"
cortex-m-rt = "0.7"
ht32f1yyy-hal = {version = "0.1.0", features = ["ht32f1755","rt","critical-section-impl"]}
```

Copy over the [memory.x](memory.x) file to your project and
uncomment the correct entry for your device.

If you are unfamiliar with embedded development using Rust, there are
a number of fantastic resources available to help.

- [Embedded Rust Documentation](https://docs.rust-embedded.org/)
- [The Embedded Rust Book](https://docs.rust-embedded.org/book/)
- [Rust Embedded FAQ](https://docs.rust-embedded.org/faq.html)
- [rust-embedded/awesome-embedded-rust](https://github.com/rust-embedded/awesome-embedded-rust)

Minimum supported Rust version
------------------------------

The Minimum Supported Rust Version (MSRV) at the moment is **1.69.0**. Older
versions **may** compile, especially when some features are not used in your
application.

License
-------

MIT License, see [LICENSE](LICENSE) or http://opensource.org/licenses/MIT for more details.

[ht32f1yyy]: https://crates.io/crates/ht32f1yyy
[ht32f5xxxx-hal]: https://github.com/ht32-rs/ht32f5xxxx-hal
[stm32-rs]: https://github.com/stm32-rs
[embedded-hal]: https://github.com/rust-embedded/embedded-hal
