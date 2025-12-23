# Unipred

This project demonstrates a hybrid Rust/Python architecture similar to Pydantic V2.

## Structure

- **python/unipred**: The high-level Python API.
- **unipred-core**: Pure Rust domain logic. Publishable to crates.io.
- **unipred-py**: Rust-to-Python bindings using PyO3. Used to build the Python extension.

## Development

### Prerequisites

- Rust (cargo)
- Python >= 3.8
- maturin (`pip install maturin`)

### Building

To build the Python package with the Rust extension:

```bash
maturin develop
```

This installs the package into your current virtual environment.

### Testing

**Rust:**

```bash
cargo test
```

**Python:**

```bash
python -c "import unipred; print(unipred.hello())"
```

## Publishing

- **PyPI**: Use `maturin publish`.
- **Crates.io**: Use `cargo publish -p unipred-core`.
