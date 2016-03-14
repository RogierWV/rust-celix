# rust-celix
Rust bindings for Apache Celix. Provides bindings for Celix and a build tool for compiling and packing a Celix bundle.

rustdoc available at http://rogierwv.github.io/celix-rust/celix.

## Requirements
* Linux (might work on other Unix(like) systems, but not tested)
* Celix installed (currently only in `/usr/local`)
* `zip` command available on `$PATH`
* `/tmp` available
* git
* Rust (only tested on nightly)
* Cargo

## Installation
```bash
$ git clone https://github.com/RogierWV/rust-celix.git
$ cd rust-celix
$ cargo install
```

## Usage
* Create a new cargo project using `cargo new`
* Add dependency to this repository `celix = { git = "https://github.com/RogierWV/rust-celix.git" }`
* Compile using `cargo celix`
