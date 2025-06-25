# leaktracer

[![license-mit](https://img.shields.io/crates/l/leaktracer.svg)](https://opensource.org/licenses/MIT)
[![repo-stars](https://img.shields.io/github/stars/veeso/leaktracer?style=flat)](https://github.com/veeso/leaktracer/stargazers)
[![downloads](https://img.shields.io/crates/d/leaktracer.svg)](https://crates.io/crates/leaktracer)
[![latest-version](https://img.shields.io/crates/v/leaktracer.svg)](https://crates.io/crates/leaktracer)
[![ko-fi](https://img.shields.io/badge/donate-ko--fi-red)](https://ko-fi.com/veeso)
[![conventional-commits](https://img.shields.io/badge/Conventional%20Commits-1.0.0-%23FE5196?logo=conventionalcommits&logoColor=white)](https://conventionalcommits.org)

[![lib-ci](https://github.com/veeso/leaktracer/actions/workflows/cargo.yml/badge.svg)](https://github.com/veeso/leaktracer/actions)
[![coveralls](https://coveralls.io/repos/github/veeso/leaktracer/badge.svg)](https://coveralls.io/github/veeso/leaktracer)
[![docs](https://docs.rs/leaktracer/badge.svg)](https://docs.rs/leaktracer)

---

- [leaktracer](#leaktracer)
  - [Introduction](#introduction)
    - [Why do I need this?](#why-do-i-need-this)
  - [Usage](#usage)
    - [Cargo.toml](#cargotoml)
    - [Setup](#setup)
    - [Accessing the stats](#accessing-the-stats)
  - [Example](#example)
  - [Debug only](#debug-only)
  - [Support the developer](#support-the-developer)
  - [Changelog](#changelog)
  - [License](#license)

---

## Introduction

A Rust allocator to trace memory allocations in Rust programs, by intercepting the allocations.

The library provides the `LeaktracerAllocator`, which is an allocator that for each allocation, it stores the memory allocated and the allocation count for each function that allocated memory.

It's extremely easy to setup and it was designed to have something really **plug-and-play**.

### Why do I need this?

You may ask why you would need this library in a language like Rust, which is known for its memory safety. The answer is that even in Rust, memory leaks can occur, especially when storing data in maps or vectors along time without cleaning them up.

Sometimes it can happen that you don't know where the huge memory usage is coming from, because either the cleanup method is not working, or you forgot to clean up the data. In complex applications, this can be a nightmare to debug, so that's why I created this library.

## Usage

### Cargo.toml

Add the following to your `Cargo.toml`:

```toml
[dependencies]
leaktracer = "0.1"
```

### Setup

Then, in your `main.rs` you need to **set the allocator** to `LeaktracerAllocator` and **initialize the symbol table**:

```rust
use leaktracer::LeaktracerAllocator;

#[global_allocator]
static ALLOCATOR: LeaktracerAllocator = LeaktracerAllocator::init();
```

and then at the beginning of your `main` function, initialize the symbol table:

```rust
leaktracer::init_symbol_table(&["my_crate_name"]);
```

The `init_symbol_table` function takes a slice of strings, which are the names of the crates you want to trace. This is useful if you have multiple crates in your project and you want to trace only specific ones.

Why is this necessary? Because the library use the `backtrace` to get the current call stack, but unfortunately the backtrace, is quite *polluted* by other non-relevant calls (such as `std::alloc`, `std::vec`, etc.), so you need to specify which crates you want to trace.

### Accessing the stats

Of course, once initialized you want to access the stats, to see how many allocations were made, and where they were made.

You can do this by accessing the `symbol_table` like this:

```rust
leaktracer::with_symbol_table(|table| {
    for (name, symbol) in table.iter() {
        println!(
            "Symbol: {name}, Allocated: {}, Count: {}",
            symbol.allocated(),
            symbol.count()
        );
    }
})?;
```

## Example

You can find an example in the `examples` folder at `examples/tracing.rs`.

You can run the example with:

```bash
cargo run --example tracing
```

## Debug only

The `LeaktracerAllocator` is meant to be used in debug mode only, as it uses the `backtrace` crate to get the call stack, which is not available in release mode and it's extremely slow and expensive. Therefore, it is recommended to use this library only in debug mode, to trace memory allocations and leaks during development.

## Support the developer

If you like **maybe-fut**, please consider a little donation 🥳

[![ko-fi](https://img.shields.io/badge/Ko--fi-F16061?style=for-the-badge&logo=ko-fi&logoColor=white)](https://ko-fi.com/veeso)
[![PayPal](https://img.shields.io/badge/PayPal-00457C?style=for-the-badge&logo=paypal&logoColor=white)](https://www.paypal.me/chrisintin)

---

## Changelog

[View Changelog here](CHANGELOG.md)

---

## License

Licensed under MIT license ([SEE LICENSE](LICENSE) or <http://opensource.org/licenses/MIT>)
