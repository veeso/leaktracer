#![crate_name = "leaktracer"]
#![crate_type = "lib"]

//! # Leaktracer
//!
//! A Rust allocator to trace memory allocations in Rust programs, by intercepting the allocations.
//!
//! ## Introduction
//!
//! A Rust allocator to trace memory allocations in Rust programs, by intercepting the allocations.
//!
//! The library provides the `LeaktracerAllocator`, which is an allocator that for each allocation,
//! it stores the memory allocated and the allocation count for each function that allocated memory.
//!
//! It's extremely easy to setup and it was designed to have something really **plug-and-play**.
//!
//! ### Why do I need this?
//!
//! You may ask why you would need this library in a language like Rust, which is known for its memory safety.
//! The answer is that even in Rust, memory leaks can occur,
//! especially when storing data in maps or vectors along time without cleaning them up.
//!
//! Sometimes it can happen that you don't know where the huge memory usage is coming from,
//! because either the cleanup method is not working, or you forgot to clean up the data.
//!
//! In complex applications, this can be a nightmare to debug, so that's why I created this library.
//!
//! ## Usage
//!
//! ### Cargo.toml
//!
//! Add the following to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! leaktracer = "0.1"
//! ```
//!
//! ### Setup
//!
//! Then, in your `main.rs` you need to **set the allocator** to [`LeaktracerAllocator`] and **initialize the symbol table**:
//!
//! ```rust
//! use leaktracer::LeaktracerAllocator;
//!
//! #[global_allocator]
//! static ALLOCATOR: LeaktracerAllocator = LeaktracerAllocator::init();
//! ```
//!
//! and then at the beginning of your `main` function, initialize the symbol table:
//!
//! ```rust
//! leaktracer::init_symbol_table(&["my_crate_name"]);
//! ```
//!
//! The [`crate::init_symbol_table`] function takes a slice of strings, which are the names of the crates you want to trace.
//! This is useful if you have multiple crates in your project and you want to trace only specific ones.
//!
//! Why is this necessary? Because the library use the `backtrace` to get the current call stack, but unfortunately the backtrace,
//! is quite *polluted* by other non-relevant calls (such as [`std::alloc`], [`std::vec`], etc.),
//! so you need to specify which crates you want to trace.
//!
//! ### Accessing the stats
//!
//! Of course, once initialized you want to access the stats, to see how many allocations were made, and where they were made.
//!
//! You can do this by accessing the `symbol_table` using the [`leaktracer::with_symbol_table`] function, like this:
//!
//! ```rust
//! leaktracer::init_symbol_table(&["my_crate_name"]);
//!
//! leaktracer::with_symbol_table(|table| {
//!     for (name, symbol) in table.iter() {
//!         println!(
//!             "Symbol: {name}, Allocated: {}, Count: {}",
//!             symbol.allocated(),
//!             symbol.count()
//!         );
//!     }
//! }).expect("Failed to access symbol table");
//! ```
//!
//! You can also access the full amount of memory allocated and the total count of allocations by using the [`LeaktracerAllocator`] methods:
//!
//! ```rust
//! use leaktracer::LeaktracerAllocator;
//!
//! #[global_allocator]
//! static ALLOCATOR: LeaktracerAllocator = LeaktracerAllocator::init();
//!
//! fn main() {
//!     leaktracer::init_symbol_table(&["my_crate_name"]);
//!
//!     println!(
//!         "Allocated {} bytes",
//!         ALLOCATOR.allocated()
//!     );
//! }
//! ```
//!
//! ## Example
//!
//! ```rust
//! use leaktracer::LeaktracerAllocator;
//!
//! #[global_allocator]
//! static ALLOCATOR: LeaktracerAllocator = LeaktracerAllocator::init();
//!
//! fn main() {
//!    // Initialize the symbol table with the current crate's symbols
//!    leaktracer::init_symbol_table(&["my_crate_name"]);
//!
//!    leaktracer::with_symbol_table(|table| {
//!        for (name, symbol) in table.iter() {
//!            println!(
//!                "Symbol: {name}, Allocated: {}, Count: {}",
//!                symbol.allocated(),
//!                symbol.count()
//!            );
//!        }
//!    }).expect("Failed to access symbol table");
//! }
//! ```
//!
//! ## Debug only
//!
//! The [`LeaktracerAllocator`] is meant to be used in debug mode only, as it uses the `backtrace` crate
//! to get the call stack, which is not available in release mode and it's extremely slow and expensive.
//!
//! Therefore, it is possible to use the library only in debug mode.
//!

#![doc(html_playground_url = "https://play.rust-lang.org")]

mod alloc;
mod symbols;

pub use self::alloc::{LeaktracerAllocator, init_symbol_table, with_symbol_table};
pub use self::symbols::{Symbol, SymbolTable};
