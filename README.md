# ADVERTISEMENT

This is a **PROTOTYPE**. It has several parsing and transpilation errors that will cause real OpenGL or DirectX compilers to crash,
and which will hopefully be fixed in the future by the creator or the community. Features such as *mathematical functions, operators
attached to literals, or array initialization* are **known limitations**, as they may not parse correctly or may even cause a crash
(for the compiler, because I doubt the transpiler will crash since it was written in **Rust**, but there are no guarantees).
Thank you for your understanding. <3

# Clock

Clock is an experimental shader language and transpilation pipeline written in Rust.

It reads source files written in CLKIL, Clock's C#-style shading language, parses them into a shared AST, and sends that AST to a
graphics API backend plugin. The current backends target OpenGL 3.3 and OpenGL ES 3.0.

## Project Status

Clock is early-stage software. The frontend, AST ABI, and OpenGL backend are actively evolving, and the current language support is
intentionally small.

At the moment, the implemented graphics backends are:

- `api/OpenGL/3.3`
- `api/OpenGLES/3.0`

Other API/version folders may exist for future work, but they are not complete targets unless their backend crate implements the
required plugin entry point.

## Repository Layout

```text
.
|-- core/                 # Clock CLI, lexer, parser, AST ABI, plugin loader
|-- api/OpenGL/3.3/       # OpenGL 3.3 backend plugin
|-- api/OpenGLES/3.0/     # OpenGL ES 3.0 backend plugin
|-- tests/                # Example CLKIL files and generated GLSL outputs
|-- triangle.clk          # Basic sample shader
|-- DOCUMENTACION.md      # Original Spanish syntax documentation
|-- README.md             # Project overview
|-- CONTRIBUTING.md       # Contribution guide
`-- SYNTAX.md             # CLKIL syntax reference
```

## How It Works

```text
.clk source
  -> lexer
  -> parser
  -> ClockAST
  -> graphics API plugin
  -> generated shader source
```

The `core` crate owns the CLI, tokenization, parsing, AST memory layout, and dynamic plugin loading. Backend plugins expose a `compile_ast`
function through a C-compatible ABI and return a `TranspiledPipeline` containing generated shader stages.

The current OpenGL-family plugins emit:

- `*.vert`
- `*.frag`

## Requirements

- Rust 2021 toolchain
- Cargo
- A platform that supports dynamic libraries
- Optional: `make`, if you want to use the provided `Makefile`

## Build

Build the full workspace:

```bash
cargo build --release
```

Or use the Makefile:

```bash
make
```

The Makefile builds the workspace and copies plugin artifacts into a release API layout.

## Usage

Run the Clock CLI with a source file, API name, and API version:

```bash
cargo run -p Clock -- file=triangle.clk api=OpenGL version=3.3
```

For OpenGL ES 3.0:

```bash
cargo run -p Clock -- file=triangle.clk "api=OpenGL ES" version=3.0
```

The compact backend ID also works: `api=OpenGLES`.

With a release build:

```bash
./target/release/Clock file=triangle.clk api=OpenGL version=3.3
```

On success, Clock writes generated shader files next to the input file. For `triangle.clk`, the outputs are:

```text
triangle.vert
triangle.frag
```

## Example

```clock
public struct VertexOutput
{
    [Builtin(BuiltinType.Position)]
    public float4 Position;

    [Location(0)]
    public float3 Color;
}

public class TriangleShader
{
    [Vertex]
    public VertexOutput MainVertex()
    {
        return new VertexOutput
        {
            Position = new float4(0.0f, 0.0f, 0.0f, 1.0f),
            Color = new float3(1.0f, 0.0f, 0.0f)
        };
    }

    [Fragment]
    public float4 MainFragment(VertexOutput input)
    {
        return new float4(input.Color, 1.0f);
    }
}
```

See `SYNTAX.md` for the current CLKIL syntax.

## License

The source files include Apache License 2.0 headers.
