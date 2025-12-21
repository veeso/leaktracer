# Changelog

## 0.1.6

Released on 2025-12-20

- [Issue #2](https://github.com/veeso/leaktracer/issues/2): Correctly trace deallocations by storing a map between allocation IDs and their allocation symbol location.
  - This has also improved performance a lot by avoiding symbol resolution during deallocation, so it should be basically x2 faster now.
- Fixed leaktracer not working on MacOS due to how the allocation works there.

## 0.1.5

Released on 2025-12-11

- Prevent underflow in allocated bytes counter during deallocation.

## 0.1.4

Released on 2025-06-26

- Prevent allocations during lock acquisition to avoid deadlocks.

## 0.1.1

Released on 2025-06-26

- Better documentation

## 0.1.0

Released on 2025-06-25

- Initial release of the project.
