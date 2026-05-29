If you want to add a new API and/or Version, follow these rules:

1° It must be a DLL, SO or DYLIB.
2° If it's written in another language than C, ALWAYS use extern "C" (Clock is made in Rust).
3° The plugin must export a `compile_ast` function with signature:
   `unsafe extern "C" fn(ast: *const ClockAST) -> TranspiledPipeline`
4° The plugin must export `free_transpiled_pipeline` and properly free any returned strings.
5° Validate input pointers, and do not panic on invalid AST data. Return null shader pointers on failure.
6° Each API backend should emit a signature comment at the top of generated shaders:
   `// This shader was made with Clock`
7° Each backend must enforce version-specific feature limits.
   - For example, `api/OpenGL/3.3` must reject `[Tessellation]` and `[Compute]` because GLSL 3.3 core does not support them.
8° Do not bake API-specific shader names into the core AST. Use abstract builtins like `Position`, `VertexId`, `InstanceId`, etc.
9° Add example `.clk` files under `tests/` when you add or change a backend implementation.
10° Document any API-specific behavior or compatibility requirements in this file so other plugin authors do not break the runtime.
