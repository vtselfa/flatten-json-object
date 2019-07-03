[![](https://camo.githubusercontent.com/2fee3780a8605b6fc92a43dab8c7b759a274a6cf/68747470733a2f2f696d672e736869656c64732e696f2f62616467652f72757374632d737461626c652d627269676874677265656e2e737667)](https://www.rust-lang.org/downloads.html)
[![](https://travis-ci.org/durch/rust-flatten-json.svg?branch=master)](https://travis-ci.org/durch/rust-flatten-json)
[![](http://meritbadge.herokuapp.com/rust-flatten-json)](https://crates.io/crates/rust-flatten-json)
![](https://img.shields.io/crates/d/rust-flatten-json.svg)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/durch/rust-flatten-json/blob/master/LICENSE.md)
## rust-flatten-json [[docs](https://durch.github.io/rust-flatten-json/)]

Tiny Rust library for flattening JSON and infering JSON types

### Usage

*In your Cargo.toml*

```
[dependencies]
rust-flatten-json = "0.1.0"
```

#### Example

Read `JSON` from `stdin` and output flat `JSON` to `stdout` -> [bin/from_stdin.rs](https://github.com/durch/rust-flatten-json/blob/master/src/bin/from_stdin.rs)

