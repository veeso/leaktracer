//! A Rust allocator to trace memory allocations in Rust programs, by intercepting the allocations.
//!
//! ## Example
//!
//! ```rust
//! use leaktracer::LeaktracerAllocator;
//!
//! #[global_allocator]
//! static ALLOCATOR: LeaktracerAllocator = LeaktracerAllocator::init();
//! ```

mod alloc;

pub use self::alloc::LeaktracerAllocator;
