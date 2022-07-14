# Setup RTIC on nrf52 with low power RTC monotonic

Assumes some cargo tools to be installed (e.g. cargo-edit and cargo generate), as well as the correct rust (Cortex-M) target on nightly toolchain.

## Install knurling rs template an adjust for use

Get the template project

	$ cargo generate --git https://github.com/knurling-rs/app-template --branch main --name rtic-nrf-rtc

Use the nightly toolchain

	$ rustup default nightly

edit `./.cargo/config.toml`

in my case:

```
[target.'cfg(all(target_arch = "arm", target_os = "none"))']
runner = "probe-run --chip nRF52833_xxAA"

rustflags = [
  "-C", "linker=flip-link",
  "-C", "link-arg=-Tlink.x",
  "-C", "link-arg=-Tdefmt.x",
  # This is needed if your flash or ram addresses are not aligned to 0x10000 in memory.x
  # See https://github.com/rust-embedded/cortex-m-quickstart/pull/95
  "-C", "link-arg=--nmagic",
]

[build]
target = "thumbv7em-none-eabihf" # Cortex-M4F and Cortex-M7F (with FPU)

[alias]
rb = "run --bin"
rrb = "run --release --bin"
```

Add correct hal to `Cargo.toml` and in `lib.rs`. In my case:

	$ cargo add nrf52833-hal

in `lib.rs`

    pub use nrf52833_hal as hal;

Now you should be able to build and run the examples, e.g.

	$ cargo run --bin hello

## Remove examples and add RTIC code

Steps to include RTIC/RTC support (done in the repo already)

	$ cargo add cortex-m-rtic
	$ cargo add rtic_monotonic
	$ cargo add fugit
	
And see the RTIC/RTC example files added in the repo

	src/bin/rtic_rtc.rs
	src/monotonic_nrf52_rtc.rs

Run RTC example

	$ DEFMT_LOG=debug cargo run --bin rtic_rtc

