/*
  Copyright 2026 GeraPro2_0

  Licensed under the Apache License, Version 2.0 (the "License");
  you may not use this file except in compliance with the License.
  You may obtain a copy of the License at

  http://www.apache.org/licenses/LICENSE-2.0

  Unless required by applicable law or agreed to in writing, software
  distributed under the License is distributed on an "AS IS" BASIS,
  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
  See the License for the specific language governing permissions and
  limitations under the License.
*/

use clock_core::ast::{CMethod, ClockAST, Decorator, TranspiledPipeline};
use std::ffi::{CStr, CString};

/// # Safety
///
/// `ast_ptr` debe ser un puntero válido a un `ClockAST` creado y llenado por el runtime,
/// y no debe apuntar a memoria liberada previamente.
#[no_mangle]
pub unsafe extern "C" fn compile_ast(ast_ptr: *const ClockAST) -> TranspiledPipeline {
    if ast_ptr.is_null() {
        return TranspiledPipeline {
            vertex_source: std::ptr::null_mut(),
            fragment_source: std::ptr::null_mut(),
        };
    }

    let ast = &*ast_ptr;
    let mut vert_output = String::new();
    let mut frag_output = String::new();

    if let Err(reason) = validate_feature_support(ast) {
        eprintln!("[Clock OpenGL ES 3.0] Transpilation canceled: {}", reason);
        return TranspiledPipeline {
            vertex_source: std::ptr::null_mut(),
            fragment_source: std::ptr::null_mut(),
        };
    }

    vert_output.push_str("// This shader was made with Clock\n");
    vert_output.push('\n');
    vert_output.push_str("#version 300 es\n");
    vert_output.push_str("precision highp float;\n");
    vert_output.push_str("precision highp int;\n");
    vert_output.push('\n');
    frag_output.push_str("// This shader was made with Clock\n");
    frag_output.push('\n');
    frag_output.push_str("#version 300 es\n");
    frag_output.push_str("precision highp float;\n");
    frag_output.push_str("precision highp int;\n");
    frag_output.push('\n');

    if !ast.structures.is_null() && ast.structures_count > 0 {
        let structures = std::slice::from_raw_parts(ast.structures, ast.structures_count as usize);
        for r_struct in structures {
            if !r_struct.fields.is_null() && r_struct.fields_count > 0 {
                let fields =
                    std::slice::from_raw_parts(r_struct.fields, r_struct.fields_count as usize);
                for field in fields {
                    let field_name = CStr::from_ptr(field.name).to_string_lossy().into_owned();
                    let field_type = CStr::from_ptr(field.data_type)
                        .to_string_lossy()
                        .into_owned();

                    if field.decorators_count > 0 && !field.decorators.is_null() {
                        let decorators = std::slice::from_raw_parts(
                            field.decorators,
                            field.decorators_count as usize,
                        );
                        for dec in decorators {
                            if let Decorator::Location(loc) = dec {
                                vert_output.push_str(&format!(
                                    "layout (location = {}) out {} {};\n",
                                    loc,
                                    map_type(&field_type),
                                    field_name
                                ));
                                frag_output.push_str(&format!(
                                    "layout (location = {}) in {} {};\n",
                                    loc,
                                    map_type(&field_type),
                                    field_name
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    frag_output.push_str("\nout vec4 FragColor;\n");

    if !ast.classes.is_null() && ast.classes_count > 0 {
        let classes = std::slice::from_raw_parts(ast.classes, ast.classes_count as usize);
        for class in classes {
            if !class.methods.is_null() && class.methods_count > 0 {
                let methods =
                    std::slice::from_raw_parts(class.methods, class.methods_count as usize);
                for method in methods {
                    let body_content = if !method.body_raw.is_null() {
                        CStr::from_ptr(method.body_raw)
                            .to_string_lossy()
                            .into_owned()
                    } else {
                        String::new()
                    };

                    if is_method_decorated_as(method, Decorator::Vertex) {
                        vert_output.push_str("\nvoid main() {\n");
                        if !body_content.is_empty() {
                            let glsl_body = body_content
                                .replace("float4", "vec4")
                                .replace("float3", "vec3")
                                .replace("float2", "vec2")
                                .replace("new float4", "vec4")
                                .replace("new float3", "vec3")
                                .replace("new float2", "vec2")
                                .replace("vec2 [ ]", "vec2[]")
                                .replace("vec3 [ ]", "vec3[]")
                                .replace("vIdx", "gl_VertexID");

                            let glsl_body = translate_array_initializers(&glsl_body);
                            let glsl_body = translate_intrinsics(&glsl_body);
                            let glsl_body = translate_vertex_output_return(&glsl_body);
                            let glsl_body = strip_numeric_f_suffix(&glsl_body);

                            if !glsl_body.trim().is_empty() {
                                for line in glsl_body.lines() {
                                    let line = line.trim_end();
                                    if !line.trim().is_empty() {
                                        vert_output.push_str("    ");
                                        vert_output.push_str(line);
                                        vert_output.push('\n');
                                    }
                                }
                            }
                        }
                        vert_output.push_str("}\n");
                    } else if is_method_decorated_as(method, Decorator::Fragment) {
                        frag_output.push_str("\nvoid main() {\n");
                        if !body_content.is_empty() {
                            frag_output.push_str("    FragColor = vec4(Color, 1.0);\n");
                        }
                        frag_output.push_str("}\n");
                    }
                }
            }
        }
    }

    TranspiledPipeline {
        vertex_source: CString::new(vert_output).unwrap().into_raw(),
        fragment_source: CString::new(frag_output).unwrap().into_raw(),
    }
}

unsafe fn is_method_decorated_as(method: &CMethod, target: Decorator) -> bool {
    if method.decorators.is_null() || method.decorators_count == 0 {
        return false;
    }
    let decorators =
        std::slice::from_raw_parts(method.decorators, method.decorators_count as usize);
    decorators
        .iter()
        .any(|dec| std::mem::discriminant(dec) == std::mem::discriminant(&target))
}

fn validate_feature_support(ast: &ClockAST) -> Result<(), &'static str> {
    if !ast.classes.is_null() && ast.classes_count > 0 {
        let classes =
            unsafe { std::slice::from_raw_parts(ast.classes, ast.classes_count as usize) };
        for class in classes {
            if class.methods.is_null() || class.methods_count == 0 {
                continue;
            }
            let methods =
                unsafe { std::slice::from_raw_parts(class.methods, class.methods_count as usize) };
            for method in methods {
                if unsafe { is_method_decorated_as(method, Decorator::Tessellation) } {
                    return Err("[Tessellation] is not supported in OpenGL ES 3.0.");
                }
                if unsafe { is_method_decorated_as(method, Decorator::Compute) } {
                    return Err("[Compute] is not supported in OpenGL ES 3.0. Use OpenGL ES 3.1 or newer for compute shaders.");
                }
            }
        }
    }
    Ok(())
}

fn map_type(clock_type: &str) -> &str {
    match clock_type {
        "float4" => "vec4",
        "float3" => "vec3",
        "float2" => "vec2",
        _ => "vec3",
    }
}
fn translate_array_initializers(body: &str) -> String {
    let mut transformed = body
        .replace("new float4[]", "vec4[]")
        .replace("new float3[]", "vec3[]")
        .replace("new float2[]", "vec2[]")
        .replace("new float4", "vec4")
        .replace("new float3", "vec3")
        .replace("new float2", "vec2")
        .replace("float4", "vec4")
        .replace("float3", "vec3")
        .replace("float2", "vec2")
        .replace("new vec4", "vec4")
        .replace("new vec3", "vec3")
        .replace("new vec2", "vec2")
        .replace("vec2 [ ]", "vec2[]")
        .replace("vec3 [ ]", "vec3[]")
        .replace("vec2 []", "vec2[]")
        .replace("vec3 []", "vec3[]")
        .replace("new vec2 [ ]", "new vec2[]")
        .replace("new vec3 [ ]", "new vec3[]")
        .replace("new vec2 []", "new vec2[]")
        .replace("new vec3 []", "new vec3[]")
        .replace("vec2[]positions", "vec2[] positions")
        .replace("vec3[]colors", "vec3[] colors");

    transformed = normalize_array_spacing(&transformed);
    transformed = rewrite_array_declarations(transformed, "vec2");
    transformed = rewrite_array_declarations(transformed, "vec3");
    transformed
}

fn translate_intrinsics(body: &str) -> String {
    let mut transformed = body.to_string();

    for (source, replacement) in [
        ("Math.Sin(", "sin("),
        ("Math.Cos(", "cos("),
        ("Math.Tan(", "tan("),
        ("Math.Asin(", "asin("),
        ("Math.Acos(", "acos("),
        ("Math.Atan(", "atan("),
        ("Math.Sqrt(", "sqrt("),
        ("Math.Pow(", "pow("),
        ("Math.Exp(", "exp("),
        ("Math.Log(", "log("),
        ("Math.Abs(", "abs("),
        ("Math.Min(", "min("),
        ("Math.Max(", "max("),
        ("Math.Clamp(", "clamp("),
        ("Math.Floor(", "floor("),
        ("Math.Ceil(", "ceil("),
        ("Math.Round(", "round("),
        ("Math.Frac(", "fract("),
        ("Math.Lerp(", "mix("),
        ("Vector2.Dot(", "dot("),
        ("Vector3.Dot(", "dot("),
        ("Vector4.Dot(", "dot("),
        ("Vector2.Cross(", "cross("),
        ("Vector3.Cross(", "cross("),
        ("Vector4.Cross(", "cross("),
        ("Vector2.Normalize(", "normalize("),
        ("Vector3.Normalize(", "normalize("),
        ("Vector4.Normalize(", "normalize("),
        ("Vector2.Distance(", "distance("),
        ("Vector3.Distance(", "distance("),
        ("Vector4.Distance(", "distance("),
        ("Vector3.Reflect(", "reflect("),
        ("Vector3.Refract(", "refract("),
        ("Vector3.FaceForward(", "faceforward("),
        ("Vector2.Lerp(", "mix("),
        ("Vector3.Lerp(", "mix("),
        ("Vector4.Lerp(", "mix("),
    ] {
        transformed = transformed.replace(source, replacement);
    }

    transformed
}

fn normalize_array_spacing(body: &str) -> String {
    let mut output = String::with_capacity(body.len());
    let chars: Vec<char> = body.chars().collect();
    let mut idx = 0;

    while idx + 1 < chars.len() {
        if chars[idx] == '[' && chars[idx + 1] == ']' {
            output.push('[');
            output.push(']');
            if let Some(next) = chars.get(idx + 2) {
                if next.is_alphanumeric() || *next == '_' {
                    output.push(' ');
                }
            }
            idx += 2;
        } else {
            output.push(chars[idx]);
            idx += 1;
        }
    }

    if idx < chars.len() {
        output.push(chars[idx]);
    }

    output
}

fn rewrite_array_declarations(mut body: String, element_type: &str) -> String {
    let search = format!("{}[]", element_type);
    let mut cursor = 0;

    while let Some(position) = body[cursor..].find(&search).map(|idx| idx + cursor) {
        if is_preceded_by_equal(&body, position) {
            cursor = position + search.len();
            continue;
        }

        if let Some((end, replacement)) = build_array_replacement(&body, position, element_type) {
            body.replace_range(position..end, &replacement);
            cursor = position + replacement.len();
        } else {
            cursor = position + search.len();
        }
    }

    body
}

fn is_preceded_by_equal(body: &str, position: usize) -> bool {
    let bytes = body.as_bytes();
    let mut idx = position;

    while idx > 0 {
        idx -= 1;
        let ch = bytes[idx];
        if ch.is_ascii_whitespace() {
            continue;
        }
        return ch == b'=';
    }

    false
}

fn build_array_replacement(
    body: &str,
    position: usize,
    element_type: &str,
) -> Option<(usize, String)> {
    let bytes = body.as_bytes();
    let mut idx = position + element_type.len() + 2;
    let len = bytes.len();

    while idx < len && bytes[idx].is_ascii_whitespace() {
        idx += 1;
    }

    let var_start = idx;
    while idx < len && !bytes[idx].is_ascii_whitespace() && bytes[idx] != b'=' {
        idx += 1;
    }

    let var_name = body[var_start..idx].trim();
    if var_name.is_empty() {
        return None;
    }

    while idx < len && bytes[idx].is_ascii_whitespace() {
        idx += 1;
    }

    if idx >= len || bytes[idx] != b'=' {
        return None;
    }
    idx += 1;

    while idx < len && bytes[idx].is_ascii_whitespace() {
        idx += 1;
    }

    let init_start = idx;
    let mut depth = 0;
    let mut end = idx;
    while end < len {
        match bytes[end] {
            b'{' => depth += 1,
            b'}' => {
                if depth > 0 {
                    depth -= 1;
                }
            }
            b';' if depth == 0 => {
                end += 1;
                break;
            }
            _ => {}
        }
        end += 1;
    }

    if end > len || end <= init_start {
        return None;
    }

    let raw_initializer = body[init_start..end - 1].trim();
    let element_count = count_array_elements(raw_initializer, element_type).max(1);
    let initializer = rewrite_initializer(raw_initializer, element_type, element_count)?;

    let replacement = format!(
        "{} {}[{}] = {};",
        element_type, var_name, element_count, initializer
    );
    Some((end, replacement))
}

fn rewrite_initializer(initializer: &str, element_type: &str, count: usize) -> Option<String> {
    let mut init = initializer.trim().to_string();

    if let Some(rest) = init.strip_prefix(element_type) {
        init = rest.trim_start().to_string();
    }

    if let Some(rest) = init.strip_prefix("[]") {
        init = rest.trim_start().to_string();
    }

    if init.starts_with('{') && init.ends_with('}') {
        let inner = init[1..init.len() - 1].trim();
        Some(format!("{}[{}]({})", element_type, count, inner))
    } else if init.starts_with('(') && init.ends_with(')') {
        Some(format!("{}[{}]{}", element_type, count, init))
    } else {
        None
    }
}

fn count_array_elements(initializer: &str, element_type: &str) -> usize {
    let mut init = initializer.trim();

    if let Some(rest) = init.strip_prefix(element_type) {
        init = rest.trim_start();
    }

    if let Some(rest) = init.strip_prefix("[]") {
        init = rest.trim_start();
    }

    if (init.starts_with('{') && init.ends_with('}'))
        || (init.starts_with('(') && init.ends_with(')'))
    {
        let inner = &init[1..init.len() - 1];
        split_top_level_commas(inner).len().max(1)
    } else {
        1
    }
}

fn translate_vertex_output_return(body: &str) -> String {
    let marker = "return new VertexOutput";
    if let Some(start) = body.find(marker) {
        let before = &body[..start];
        let rest = &body[start..];
        let open_brace = match rest.find('{') {
            Some(pos) => pos,
            None => return body.to_string(),
        };
        let close_brace = match find_matching_brace(rest, open_brace) {
            Some(pos) => pos,
            None => return body.to_string(),
        };
        let inner = &rest[open_brace + 1..close_brace];
        let suffix = &rest[close_brace + 1..];

        let mut translated = String::new();
        for field_assignment in split_top_level_commas(inner) {
            let assignment = field_assignment.trim().trim_end_matches(',').trim();
            if assignment.is_empty() {
                continue;
            }
            if let Some(eq) = assignment.find('=') {
                let field_name = assignment[..eq].trim();
                let expression = assignment[eq + 1..].trim().trim_end_matches(';').trim();
                if field_name == "Position" {
                    translated.push_str(&format!("gl_Position = {};\n", expression));
                } else {
                    translated.push_str(&format!("{} = {};\n", field_name, expression));
                }
            }
        }

        let remaining = suffix.trim_start();
        let remaining = if let Some(trimmed) = remaining.strip_prefix(';') {
            trimmed
        } else {
            remaining
        };
        return format!("{}{}{}", before, translated, remaining);
    }
    body.to_string()
}

fn strip_numeric_f_suffix(body: &str) -> String {
    let mut output = String::with_capacity(body.len());
    let chars: Vec<char> = body.chars().collect();
    let mut idx = 0;

    while idx < chars.len() {
        if chars[idx] == 'f'
            && idx > 0
            && (chars[idx - 1].is_ascii_digit() || chars[idx - 1] == '.')
        {
            idx += 1;
            continue;
        }
        output.push(chars[idx]);
        idx += 1;
    }

    output
}

fn split_top_level_commas(source: &str) -> Vec<String> {
    let mut items = Vec::new();
    let mut current = String::new();
    let mut depth = 0;

    for ch in source.chars() {
        match ch {
            '(' => {
                depth += 1;
                current.push(ch);
            }
            ')' => {
                if depth > 0 {
                    depth -= 1;
                }
                current.push(ch);
            }
            ',' if depth == 0 => {
                items.push(current.trim().to_string());
                current.clear();
            }
            other => {
                current.push(other);
            }
        }
    }

    if !current.trim().is_empty() {
        items.push(current.trim().to_string());
    }

    items
}

fn find_matching_brace(source: &str, start: usize) -> Option<usize> {
    let mut depth = 0;
    for (idx, ch) in source.chars().enumerate().skip(start) {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(idx);
                }
            }
            _ => {}
        }
    }
    None
}
