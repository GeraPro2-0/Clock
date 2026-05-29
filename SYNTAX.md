# CLKIL Syntax

CLKIL is Clock's shader language. It uses a C#-style surface syntax to describe GPU shader interfaces and entry points.

This document describes the syntax currently supported by the parser and the OpenGL-family backends.

## File Structure

A CLKIL file must define at least:

- one `struct`
- one `class`

The `struct` usually describes data passed between shader stages. The `class` contains shader entry point methods.

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

## Comments

Single-line comments:

```clock
// This is a comment.
```

Block comments:

```clock
/*
  This is a block comment.
*/
```

## Keywords

The lexer currently recognizes these reserved words:

- `public`
- `struct`
- `class`
- `return`
- `new`
- `uint`

Other type names, method names, field names, and intrinsic names are parsed as identifiers.

## Structs

Structs define named groups of fields.

```clock
public struct VertexOutput
{
    public float4 Position;
    public float3 Color;
}
```

The parser accepts decorators before fields:

```clock
[Builtin(BuiltinType.Position)]
public float4 Position;

[Location(0)]
public float3 Color;
```

The OpenGL-family backends use `[Location(n)]` fields as vertex-to-fragment varyings.

## Classes

Classes group shader methods.

```clock
public class TriangleShader
{
    [Vertex]
    public VertexOutput MainVertex()
    {
        // vertex shader body
    }

    [Fragment]
    public float4 MainFragment(VertexOutput input)
    {
        // fragment shader body
    }
}
```

## Methods

Method syntax:

```clock
[Decorator]
public ReturnType MethodName(parameters)
{
    statements
}
```

The parser records the raw method body and passes it to the backend. Parameter lists are currently skipped by the core parser, so backend behavior depends on the target plugin.

## Decorators

Decorators use square brackets:

```clock
[Vertex]
[Fragment]
[Location(0)]
[Builtin(BuiltinType.Position)]
```

Supported decorators are:

| Decorator | Meaning |
| --- | --- |
| `[Vertex]` | Marks a method as the vertex shader entry point. |
| `[Fragment]` | Marks a method as the fragment shader entry point. |
| `[Tessellation]` | Parsed, but rejected by the current OpenGL-family backends. |
| `[Compute]` | Parsed, but rejected by OpenGL 3.3 and OpenGL ES 3.0. |
| `[Location(n)]` | Assigns a numeric location to a field or output. |
| `[Builtin(BuiltinType.Position)]` | Maps a field to the backend position builtin. |
| `[Builtin(BuiltinType.VertexId)]` | Represents a vertex index builtin. |

## Builtin Types

Supported builtin values:

```clock
BuiltinType.Position
BuiltinType.VertexId
```

The OpenGL-family backends map vertex index usage to `gl_VertexID` when they see `vIdx` in a vertex body. Some examples also use `VertexId`; backend support for builtin naming is still evolving.

## Data Types

Common CLKIL shader types:

```clock
float
float2
float3
float4
uint
```

The OpenGL-family backends map:

| CLKIL | GLSL |
| --- | --- |
| `float2` | `vec2` |
| `float3` | `vec3` |
| `float4` | `vec4` |
| `uint` | `uint` |

## Numeric Literals

Supported numeric literal forms include:

```clock
0
1
-1
0.5
-0.5
1.0f
```

The OpenGL backend strips the `f` suffix in generated GLSL.

## Object Construction

Vector construction uses `new`:

```clock
new float2(0.0f, 0.5f)
new float3(1.0f, 0.0f, 0.0f)
new float4(position, 0.0f, 1.0f)
```

Struct returns can use object initializer syntax:

```clock
return new VertexOutput
{
    Position = new float4(positions[vIdx], 0.0f, 1.0f),
    Color = colors[vIdx]
};
```

## Arrays

Array declarations are used in the examples:

```clock
float2[] positions = new float2[] {
    new float2(0.0f, 0.5f),
    new float2(-0.5f, -0.5f),
    new float2(0.5f, -0.5f)
};
```

The OpenGL-family backends rewrite supported `float2`, `float3`, and `float4` array construction into GLSL-style vector arrays.

## Math Intrinsics

The OpenGL-family backends translate selected intrinsic calls to GLSL functions.

| CLKIL | GLSL |
| --- | --- |
| `Math.Sin` | `sin` |
| `Math.Cos` | `cos` |
| `Math.Tan` | `tan` |
| `Math.Sqrt` | `sqrt` |
| `Math.Pow` | `pow` |
| `Math.Abs` | `abs` |
| `Math.Min` | `min` |
| `Math.Max` | `max` |
| `Math.Clamp` | `clamp` |
| `Math.Floor` | `floor` |
| `Math.Ceil` | `ceil` |
| `Math.Round` | `round` |
| `Math.Frac` | `fract` |
| `Math.Lerp` | `mix` |
| `Vector2.Dot` | `dot` |
| `Vector3.Dot` | `dot` |
| `Vector4.Dot` | `dot` |
| `Vector3.Cross` | `cross` |
| `Vector2.Normalize` | `normalize` |
| `Vector3.Normalize` | `normalize` |
| `Vector4.Normalize` | `normalize` |
| `Vector2.Distance` | `distance` |
| `Vector3.Distance` | `distance` |
| `Vector4.Distance` | `distance` |
| `Vector3.Reflect` | `reflect` |
| `Vector3.Refract` | `refract` |
| `Vector3.FaceForward` | `faceforward` |

## Current OpenGL-Family Limits

The OpenGL 3.3 and OpenGL ES 3.0 backends currently support vertex and fragment shader generation only.

It rejects:

```clock
[Tessellation]
[Compute]
```

The current backend performs several source-to-source rewrites on raw method bodies, so complex CLKIL expressions may need backend work before they transpile correctly.
