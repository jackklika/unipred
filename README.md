# Unipred

Unified python library for prediction market exchange operations.

Intended to used for low-latency applications and local-first research.

This project demonstrates a hybrid Rust/Python architecture similar to Pydantic V2.

## Goals

- **Local-first**: Give library users full control of state and data, so we can run analytical operations on local data and also respect all terms and conditions of api usage.
- **Fast**: Run operations quickly, putting as little latency as possible between the exchange and user intent.
- **Multi-language**: Requests for market data and responses are stored in protobuf format, allowing working with the same operations from rust, python, and data formats.

## High Level Structure

- Communication with exchanges happens in the rust core `clients` module
- UnipredCore is defined in rust, which is the broker of all commands and responses in the application.
- Exchange-specific operations or data is mapped to shared domain objects in rust
- Rust project at `/unipred-py` exposes internal python bindings
- `python/unipred/commands` exposes public functions which wrap the rust core, and perform the protobuf serialization/deserialization in the python runtime

## Project Structure

- **proto**: Protobuf definitions for data, requests, and responses
- **unipred-core**: Rust domain logic, including clients and mapping from exchange-specific operations to unipred domain objects.
- **unipred-py**: Python bindings for unipred-core 
- **python/unipred**: The high-level Python API, mostly wrapping the rust core.
- **context**: LLM context for development


## Development

### Prerequisites
1.  **Rust**: [Install Rust](https://www.rust-lang.org/tools/install).
2.  **uv**: [Install uv](https://github.com/astral-sh/uv).
3.  **Protocol Buffers Compiler (`protoc`)**:
    *   **macOS**: `brew install protobuf`
    *   **Ubuntu/Debian**: `apt install -y protobuf-compiler`
    *   **Windows**: [Download release](https://github.com/protocolbuffers/protobuf/releases) and add `bin` to PATH.

### Installation

1.  **Sync Python dependencies:**
    ```bash
    uv sync
    ```

2.  **Generate Python Protobuf definitions:**
    *This step generates the `market_quote_pb2.py` file required by the Python wrapper.*
    ```bash
    uv run protoc -I=proto --python_out=python/unipred proto/market_quote.proto
    ```

3.  **Build the Rust extension:**
    *This compiles the Rust code (including Rust Protobufs) and installs it into the virtual environment.*
    ```bash
    uv run maturin develop
    ```

### Configuration

Create a `.env` file in the root directory for integration tests or normal usage:

```ini
# Required for Kalshi integration tests
KALSHI_API_KEY_ID="your_key_id"
KALSHI_PRIVATE_KEY="-----BEGIN RSA PRIVATE KEY-----
...
-----END RSA PRIVATE KEY-----"
```

There is also functionality to load api key from a file.

### Running Tests

Run the full test suite:

```bash
uv run pytest
