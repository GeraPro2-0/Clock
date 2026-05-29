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

use crate::ast::{free_transpiled_pipeline, ClockAST, TranspiledPipeline};
use libloading::{Library, Symbol};
use std::ffi::CStr;
use std::fs;
use std::path::{Path, PathBuf};

// La firma DEBE retornar TranspiledPipeline, no un puntero de caracteres crudo
type CompileAstFn = unsafe extern "C" fn(ast: *const ClockAST) -> TranspiledPipeline;

struct PluginManifest {
    path: PathBuf,
    display_name: String,
}

pub fn execute_plugin(
    api: &str,
    version: &str,
    ast: &ClockAST,
) -> Result<(String, String), String> {
    let dll_path = plugin_library_path(api, version);

    if !dll_path.exists() {
        return Err(format!(
            "❌ Infrastructure error: Could not locate the graphics plugin binary at: {:?}. \nMake sure you have compiled the plugin with `cargo build --release` in the workspace.",
            dll_path
        ));
    }

    unsafe {
        let lib = Library::new(&dll_path)
            .map_err(|e| format!("Failed to load the dynamic library: {}", e))?;
        let compile_ast: Symbol<CompileAstFn> = lib
            .get(b"compile_ast\0")
            .map_err(|e| format!("Entry point 'compile_ast' not found: {}", e))?;

        let pipeline = compile_ast(ast);
        if pipeline.vertex_source.is_null() || pipeline.fragment_source.is_null() {
            return Err("The API compiler returned null pipeline stages.".to_string());
        }

        let vert_code = CStr::from_ptr(pipeline.vertex_source)
            .to_string_lossy()
            .into_owned();
        let frag_code = CStr::from_ptr(pipeline.fragment_source)
            .to_string_lossy()
            .into_owned();

        free_transpiled_pipeline(pipeline);

        Ok((vert_code, frag_code))
    }
}

fn plugin_library_path(api: &str, version: &str) -> PathBuf {
    let plugin_cargo = find_plugin_manifest(api, version)
        .map(|manifest| manifest.path)
        .unwrap_or_else(|| {
            PathBuf::from("api")
                .join(api)
                .join(version)
                .join("Cargo.toml")
        });
    let crate_name = if plugin_cargo.exists() {
        fs::read_to_string(&plugin_cargo)
            .ok()
            .and_then(|content| package_name_from_manifest(&content))
            .unwrap_or_else(|| fallback_crate_name(api, version))
    } else {
        fallback_crate_name(api, version)
    };
    let file_name = if cfg!(target_os = "windows") {
        format!("{}.dll", crate_name)
    } else if cfg!(target_os = "macos") {
        format!("lib{}.dylib", crate_name)
    } else {
        format!("lib{}.so", crate_name)
    };

    let release_path = PathBuf::from("target").join("release").join(&file_name);
    if release_path.exists() {
        return release_path;
    }

    let debug_path = PathBuf::from("target").join("debug").join(&file_name);
    if debug_path.exists() {
        return debug_path;
    }

    release_path
}

fn find_plugin_manifest(api: &str, version: &str) -> Option<PluginManifest> {
    let api_root = Path::new("api");
    for entry in fs::read_dir(api_root).ok()? {
        let api_dir = entry.ok()?.path();
        if !api_dir.is_dir() {
            continue;
        }

        let manifest = api_dir.join(version).join("Cargo.toml");
        if !manifest.exists() {
            continue;
        }

        let content = fs::read_to_string(&manifest).ok()?;
        if manifest_supports_api_name(&content, api) {
            let display_name = clock_display_name_from_manifest(&content)
                .unwrap_or_else(|| api.trim().to_string());
            return Some(PluginManifest {
                path: manifest,
                display_name,
            });
        }
    }

    None
}

pub fn backend_display_name(api: &str, version: &str) -> String {
    find_plugin_manifest(api, version)
        .map(|manifest| manifest.display_name)
        .unwrap_or_else(|| api.trim().to_string())
}

fn clock_display_name_from_manifest(content: &str) -> Option<String> {
    let mut in_clock_metadata = false;

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_clock_metadata = line == "[package.metadata.clock]";
            continue;
        }

        if in_clock_metadata && line.starts_with("display_name") {
            return value_after_equals(line);
        }
    }

    None
}

fn manifest_supports_api_name(content: &str, api: &str) -> bool {
    let requested = api.trim();
    let mut in_clock_metadata = false;

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_clock_metadata = line == "[package.metadata.clock]";
            continue;
        }

        if in_clock_metadata && line.starts_with("api_names") {
            return parse_string_array(line)
                .iter()
                .any(|name| name == requested);
        }
    }

    false
}

fn package_name_from_manifest(content: &str) -> Option<String> {
    let mut in_package = false;
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_package = line == "[package]";
        } else if in_package && line.starts_with("name") {
            return value_after_equals(line);
        }
    }

    None
}

fn parse_string_array(line: &str) -> Vec<String> {
    let Some(start) = line.find('[') else {
        return Vec::new();
    };
    let Some(end) = line.rfind(']') else {
        return Vec::new();
    };

    line[start + 1..end]
        .split(',')
        .filter_map(|item| quoted_value(item.trim()))
        .collect()
}

fn value_after_equals(line: &str) -> Option<String> {
    let eq_idx = line.find('=')?;
    quoted_value(line[eq_idx + 1..].trim())
}

fn quoted_value(value: &str) -> Option<String> {
    Some(
        value
            .trim()
            .trim_matches('"')
            .trim_matches('\'')
            .to_string(),
    )
    .filter(|value| !value.is_empty())
}

fn fallback_crate_name(api: &str, version: &str) -> String {
    format!(
        "{}_{}",
        api,
        version.replace(|c: char| !c.is_alphanumeric(), "_")
    )
}
