# Clock ⏱️

[![Rust](https://img.shields.io/badge/rust-2021-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)

Clock is an experimental shader cross-compilation pipeline and shading language runtime written in **Rust**. 

It allows developers to write shaders using **CLKIL** (a strongly-typed, object-oriented shading language heavily inspired by C# syntax), abstracting the source code into a common, shareable Abstract Syntax Tree (AST). Through a modular system of native plugins loaded via FFI, this AST is then cross-compiled into native source code for different graphics APIs.

---

## ⚠️ Project Status & Known Limitations (Help Wanted!)

Clock is currently a **Proof-of-Concept Prototype**. While the core engine successfully orchestrates token parsing, AST layout generation, and dynamic FFI plugin linkage, the string manipulation and GLSL emit phase under `api/OpenGL/3.3` is undergoing active refactoring.

If you inspect generated outputs (such as `math_functions.vert`), you will notice specific syntax bugs due to string formatting edge cases. **We value honesty and transparency, so we don't hide these defects—instead, we treat them as open milestones!**

### 🚀 Open Compiling Challenges
We are actively looking for contributors to help optimize the backend emitter inside `api/OpenGL/3.3/lib.rs` (specifically within the `clean_body_syntax` engine):
* **Lexer / Token Spacing:** Fixing operators attached directly to literals (e.g., generating `x 3.14` instead of `x * 3.14`).
* **Intrinsic Replacements:** Robust string normalization for built-in math APIs (like `Math.Sin`) when the parser separates spaces.
* **Builtin System Mappings:** Ensuring variables like `VertexId` resolve cleanly to `gl_VertexID` across nested mathematical expressions.

If you enjoy Rust AST manipulation, FFI, or graphics programming, this is a fun sandbox to jump into!

---

## 📂 Repository Layout

```text
.
|-- core/                 # Clock CLI, lexer, parser, AST ABI, plugin loader
|-- api/OpenGL/3.3/       # OpenGL 3.3 pipeline backend plugin (Active Work-In-Progress)
|-- api/OpenGLES/3.0/     # OpenGL ES 3.0 pipeline backend plugin
|-- api/DirectX/          # This doesn't exist (yet)
|-- api/Vulkan/           # This doesn't exist (yet)
|-- api/Metal/            # This doesn't exist (yet)
|-- api/WebGPU/           # This doesn't exist (yet)
|-- tests/                # Sample CLKIL files and their current generated outputs
|-- triangle.clk          # Basic sample shader source
`-- SYNTAX.md             # Detailed documentation of the CLKIL language specification
```

---

## 🔄 Architectural Workflow

```text
.clk Source ──> Lexer ──> Parser ──> ClockAST ──> FFI Plugin ──> GLSL Output (*.vert/*.frag)
```

The core module takes care of the parsing heavy-lifting and exposes a C-compatible compile_ast entry point over the ABI boundary. Backend plugins digest this raw memory mapping and return a structured TranspiledPipeline.

---

## 🚀 Quick Start
Prerequisites
Rust 2021 toolchain (Cargo).

An OS supporting compiled dynamic libraries (.dll, .so, or .dylib).

make (Optional, used to automate cross-platform artifact routing).

### 1. Build the Workspace and Organize Plugins
The repository contains a cross-platform POSIX-friendly Makefile compatible with Windows (MSYS2/Cygwin), Linux, and macOS. It builds the full workspace in release mode and automatically maps the target artifacts to their corresponding system architecture folders:

```bash
make
```

### 2. Transpile a Shader
To execute the transpiler against the sample triangle.clk file using the OpenGL 3.3 backend plugin structure, run:

```bash
./target/release/Clock file=examples/triangle.clk api=OpenGL version=3.3
```

or (depending on your OS):

```bash
./target/release/Clock file=C:/Users/Clock/examples/triangle.clk api=OpenGL version=3.3
```

---

## 📝 Code Sample (CLKIL Syntax)
```clkil
// Vertex Shader output interface and Fragment Shader input
public struct VertexOutput 
{
    [Builtin(BuiltinType.Position)] 
    public float4 Position;

    [Location(0)] 
    public float3 Color;
}

/*
  Clock Shading Pipeline (CLKIL)
  This block packages the graphical entry points.
*/
public class TriangleShader 
{
    // Vertex Shader entry point
    [Vertex]
    public VertexOutput MainVertex() 
    {
        float2[] positions = new float2[] {
            new float2(0.0f, 0.5f),
            new float2(-0.5f, -0.5f),
            new float2(0.5f, -0.5f)
        };

        float3[] colors = new float3[] {
            new float3(1.0f, 0.0f, 0.0f),
            new float3(0.0f, 1.0f, 0.0f),
            new float3(0.0f, 0.0f, 1.0f)
        };

        return new VertexOutput 
        {
            Position = new float4(positions[vIdx], 0.0f, 1.0f),
            Color = colors[vIdx]
        };
    }
}
```
### 🛹 Results (contains errors)

Vertex Shader:
```glsl
#version 330 core

layout (location = 0) out vec3 Color;

void main() {
    vec2 positions[3] = vec2[3](vec2 (0.0 , 0.5 ) , vec2 (-0.5 , -0.5 ) , vec2 (0.5 , -0.5 ));
        vec3 colors[3] = vec3[3](vec3 (1.0 , 0.0 , 0.0 ) , vec3 (0.0 , 1.0 , 0.0 ) , vec3 (0.0 , 0.0 , 1.0 ));
        gl_Position = vec4 (positions [gl_VertexID ], 0.0 , 1.0 );
    Color = colors [gl_VertexID ];
}
```

Fragment Shader:
```glsl
#version 330 core

layout (location = 0) in vec3 Color;

out vec4 FragColor;

void main() {
    FragColor = vec4(Color, 1.0);
}
```

# 📄 License
This project is licensed under the Apache License 2.0. See individual file headers for specific copyright declarations.