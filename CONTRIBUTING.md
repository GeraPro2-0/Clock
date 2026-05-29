# Contributing to Clock

Thanks for helping improve Clock. This project is still small and changing quickly, so the best contributions are focused, easy to review, and honest about what is currently implemented.

## Before You Start

Please read:

- `README.md` for the project overview
- `SYNTAX.md` for the current CLKIL language surface
- `DOCUMENTACION.md` for the original Spanish syntax notes

## Development Setup

Build the workspace:

```bash
cargo build
```

Build release artifacts:

```bash
cargo build --release
```

Run the CLI against an example:

```bash
cargo run -p Clock -- file=triangle.clk api=OpenGL version=3.3
```

Run the OpenGL ES backend:

```bash
cargo run -p Clock -- file=triangle.clk "api=OpenGL ES" version=3.0
```

The compact backend ID also works: `api=OpenGLES`.

If you use `make`, this also organizes plugin artifacts:

```bash
make
```

## Contribution Areas

Good places to contribute include:

- Lexer and parser fixes in `core/src/lexer.rs` and `core/src/parser.rs`
- AST ABI improvements in `core/src/ast.rs`
- Plugin loading behavior in `core/src/transpiler.rs`
- OpenGL 3.3 code generation in `api/OpenGL/3.3/src/lib.rs`
- OpenGL ES 3.0 code generation in `api/OpenGLES/3.0/src/lib.rs`
- New CLKIL examples in `tests/`
- Documentation updates in `README.md`, `CONTRIBUTING.md`, and `SYNTAX.md`

## Backend Plugin Contract

A graphics API backend is a Rust dynamic library loaded by the core runtime. It must export:

```rust
#[no_mangle]
pub unsafe extern "C" fn compile_ast(ast_ptr: *const ClockAST) -> TranspiledPipeline
```

The plugin receives a `ClockAST` and returns generated shader stage strings through `TranspiledPipeline`.

If you add a new backend, place it under:

```text
api/<API>/<VERSION>/
```

For example:

```text
api/OpenGL/4.3/
api/Vulkan/1.3/
```

## Coding Guidelines

- Keep changes scoped to one concern.
- Prefer existing project patterns before adding new abstractions.
- Keep the C ABI structs in `core/src/ast.rs` compatible with plugin users.
- Be careful with raw pointers and ownership across the FFI boundary.
- Avoid documenting features as supported until they work in the parser and backend.
- Add or update `.clk` examples when changing language behavior.

## Testing Changes

There is not yet a full automated test suite. For now, verify changes by building and transpiling example files:

```bash
cargo build
cargo build --release
cargo run -p Clock -- file=triangle.clk api=OpenGL version=3.3
cargo run -p Clock -- file=triangle.clk "api=OpenGL ES" version=3.0
cargo run -p Clock -- file=tests/math_functions.clk api=OpenGL version=3.3
```

Check the generated `.vert` and `.frag` files for expected output.

## Pull Request Checklist

Before submitting a change, make sure:

- The workspace builds.
- At least one relevant `.clk` file transpiles successfully.
- Documentation is updated if syntax, CLI behavior, or backend support changed.
- New backend work preserves the `compile_ast` ABI.
- Error messages are understandable to someone using the CLI.

## Style Notes

Clock currently mixes English and Spanish comments/messages. Either language is acceptable for small local changes, but new user-facing documentation should prefer clear English unless updating an existing Spanish document.
