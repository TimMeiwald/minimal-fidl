# Python bindings to Rust crates in the rest of the repo

Managed with [uv](https://docs.astral.sh/uv/). If you do not have it:

```sh
curl -LsSf https://astral.sh/uv/install.sh | sh
```

No `sudo`, and no need to create or activate a virtualenv yourself — uv makes and
manages one in `.venv/` for you.

## Working on the bindings

```sh
uv sync                    # create the environment and install dev tooling
uv run maturin develop     # compile the Rust extension into that environment
uv run pytest              # run the tests
```

Re-run `uv run maturin develop` after any change to the Rust source; `uv run
pytest` alone will keep using the previously compiled extension.

## Building a wheel to publish

```sh
uv run maturin build --release
```

## Notes

- The wheel has no runtime Python dependencies — it is the compiled extension and
  nothing else. Everything under `[dependency-groups]` in `pyproject.toml` is dev
  tooling only.
- `franca_idl.pyi` is the type stub and is still written by hand, but
  `tests/test_stub.py` now checks it against the module in both directions: a stub
  entry that does not exist fails, and so does a public member the stub omits. Add
  to the stub in the same commit as the method.
- New pyclasses produced by the `handle!` macro have to be registered by hand in
  `#[pymodule_init]` — `#[pymodule]` cannot see them. Without that they work as
  return values but cannot be imported or used with `isinstance`.
- The pyo3 version and its `abi3-py38` feature live in `Cargo.toml`.
