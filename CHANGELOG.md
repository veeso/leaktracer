# Changelog

All notable changes to this project are documented in this file.

## 0.1.6

Released on 2026-09-18

### Fixed

- deallocation done in different function can cause underflow (#3)

> - fix: Fixed deallocation trace by looking up the symbol ONLY at allocation and by storing a pointer -> Symbol table
>
> The underflow issue was already solved, this pr added a map to map the ptr to the symbol name to correctly trace the deallocation

## 0.1.5

Released on 2025-12-11

### Fixed

- Prevent underflow in allocated bytes counter during deallocation.

## 0.1.4

Released on 2025-06-26

### Added

- initial commit
- allocator
- Implementation of the allocator and of the tracing system

### Fixed

- Build only in debug
- removed release build error
- Prevent IN_ALLOC to be false during access to the symbol table
- prevent deadlocks when accessing symbols
