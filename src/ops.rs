//! Wrappers around top level `librrd` entry points.
//!
//! The approach taken by this crate is to provide a structured way of building the possibly very
//! complex invocation of `librrd` functions, which generally accept arguments as a CLI-style `argv`
//! array of strings rather than via C structs etc.
//!
//! Instead of making it the caller's responsibility to
//! properly format command strings, etc, Rust structs/enums/etc are provided which then will take
//! care of producing the appropriate strings used by `librrd`.
//!
//! The upstream `librrd` library does not itself have any applicable documentation, but the
//! upstream CLI wrappers do have docs, e.g.
//! [`rrdcreate`](https://oss.oetiker.ch/rrdtool/doc/rrdcreate.en.html), which would correspond with
//! [`create::create`]. The Rust types that generate the C arg strings have been named to match
//! those docs.

pub mod create;
pub mod fetch;
pub mod graph;
pub mod info;
pub mod update;
pub mod version;
