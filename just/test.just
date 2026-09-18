# Run all unit and documentation tests.
[group('test')]
test args="":
  cargo test --all-features {{ args }}
  cargo test --doc --all-features {{ args }}

# Generate an LCOV coverage report.
[group('test')]
coverage output="lcov.info" args="":
  cargo llvm-cov --all-features --lcov --output-path {{ output }} {{ args }}
