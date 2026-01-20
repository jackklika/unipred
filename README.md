## ⚠️ Disclaimer ⚠️ 

This is an unfinished project, and will probably stay unfinished and unmaintained.

While I think this is a good architecture (rust core with python + protobufs) for a scalable uni-market prediction market library, most of this is AI slop which is not tested to be useful. 

It was a good idea, but I cannot find a good reason to spend any more time on this. Do with it what you will.
---

# Unipred

Unified python library for prediction market exchange operations.

Intended for research and prediction market interaction with a simple, python interface across exchanges.

This project demonstrates a hybrid Rust/Python architecture similar to Pydantic V2.

## Status

Currently, this is mostly code smashed together from some kalshi and polymarket clients, some domain models from other projects, and a lot of sloppy-but-functional AI contributions. But the structure was thought out and modular, making it easy to clean up and ship for a beta 0.0.1. I want to focus on creating a solid foundation.

## High-level Goals

- **Local-first**: Give library users full control of state and data, so we can run analytical operations on local data and also respect all terms and conditions of api usage.
- **Fast & Efficient**: Run operations quickly, maximizing machine resource usage to scrape and process events. 
- **Multi-language**: Requests for market data and responses are stored in protobuf format, allowing working with the same operations from rust, python, and data formats.
- **Simple Interface**: Wrap all system commands in a lightweight python wrapper, so users can interact with the data like a data scientist in jupyter without needing to know what a tokio runtime is.

## Feature Roadmap

Here are specific features I would like to build on top of this foundation:

- **Market Correlation**: Given any market identifier, find the most correlated markets and include a 'correlation score' 
- **AI Prediction Market Tooling**: Expose plaintext instructions for how LLMs can interact with this market data, by calling python bindings, rust functions, or even just querying the datastores. Pydantic AI tools will be set up in the python module to allow safe yet automated human-in-the-loop interactions with markets, such as order flow.
- **User Interface**: A dashboard/terminal for interaction with these markets using these backend features. A sandboxed python environment could allow for scripting in the browser to set up custom workflows, perhaps using temporal.
- **Cloud hosting**: Copying exchange state to local development environments is not efficient. The exchange state and derived data like similarity embeddings would be the same, and ideally would be stored on cloud infrastructure, so we can take advantage of economies of scale. But we need to respect kalshi and polymarket data terms of service.

## High Level Structure

- Communication with exchanges happens in the rust core `clients` module
- `UnipredCore` is defined in rust, which is the broker of all commands and responses in the application.
- `unipred-core/src/commands` is where exchange-specific operations or data is mapped to shared domain objects in rust.
- `python/unipred/commands` exposes public functions which wrap the rust core, and perform the protobuf serialization/deserialization in the python runtime
- Rust project at `/unipred-py` exposes internal python bindings. Python is intended to be a thin configuration or orchestration layer, while all 'hot paths' are implemented in rust.

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
