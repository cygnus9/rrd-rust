[![](https://img.shields.io/crates/v/rrd.svg)](https://crates.io/crates/rrd) [![Docs](https://docs.rs/rrd/badge.svg)](https://docs.rs/rrd)

# rrd

Bindings to `librrd` to create and interact with round robin databases created with Tobias
Oetiker's  [`rrdtool`](https://www.rrdtool.org/).

> RRDtool is the OpenSource industry standard, high performance data logging and graphing
> system for time series data. RRDtool can be easily integrated in shell scripts, perl, python,
> ruby, lua or tcl applications.

And now also from Rust.

## Supported operations

This library provides high level APIs for the core RRD operations:

- Create - make a new round robin database (RRD)
- Update - add data to an RRD
- Fetch - get data from an RRD
- Graph - generate graphs from an RRD
- Info - get RRD metadata

There are [other operations available in the upstream `rrdtool`](https://oss.oetiker.ch/rrdtool/doc/index.en.html) (e.g.
exporting to XML, tuning parameters, etc), but as they are generally more administrative in nature, this library doesn't
expose them (yet?).

## Getting Started

Make sure `rrdtool` (or at least `librrd8`) is installed on your system.
Any reasonable package manager should have `librrd` packages available, or check [here](https://rrdtool.org/download.en.html) for instructions on building from source.

Then add `rrd` as a dependency to your project.

```toml
[dependencies]
rrd = "0.1.0"
```

### Windows

To link to `librrd-8.dll` you'll need a `.lib` file, which is not
shipped with the pre-build binaries shipped [here](https://github.com/oetiker/rrdtool-1.x/releases).

Follow these steps to create the `.lib` file:

1. Download [`librrd-8.def`](https://github.com/oetiker/rrdtool-1.x/raw/master/win32/librrd-8.def)
2. From a VS dev shell: `lib /def:librrd-8.def /out:librrd-8.lib /machine:x64`
3. Set the `LIBRRD` environment variable to the full path of `librrd-8.lib`

# License

This project is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   https://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   https://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in `rrd` by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

#### Checking with multiple OSs

Run `./scripts/check-with-different-versions.sh` to run tests in several different linux systems with varying librrd versions.