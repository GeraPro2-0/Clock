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

use clock_core::ast;

mod lexer;
mod parser;
mod transpiler;

use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::path::PathBuf;

struct ProgressBar {
    total: usize,
    current: usize,
}

impl ProgressBar {
    fn new(total: usize) -> Self {
        Self { total, current: 0 }
    }

    fn step(&mut self, label: &str) {
        self.current = (self.current + 1).min(self.total);
        self.draw(label);
    }

    fn finish(&mut self, label: &str) {
        self.current = self.total;
        self.draw(label);
        println!();
    }

    fn draw(&self, label: &str) {
        let width = 28;
        let filled = width * self.current / self.total.max(1);
        let empty = width - filled;
        let percent = self.current * 100 / self.total.max(1);

        print!(
            "\r[{}{}] {:>3}% {}",
            "#".repeat(filled),
            "-".repeat(empty),
            percent,
            label
        );
        let _ = io::stdout().flush();
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut file_path = String::new();
    let mut api = String::new();
    let mut version = String::new();

    let mut idx = 1;
    while idx < args.len() {
        let arg = &args[idx];
        if let Some(value) = arg.strip_prefix("file=") {
            file_path = value.to_string();
        } else if let Some(value) = arg.strip_prefix("api=") {
            api = value.to_string();
            while idx + 1 < args.len() && !args[idx + 1].contains('=') {
                idx += 1;
                api.push(' ');
                api.push_str(&args[idx]);
            }
        } else if let Some(value) = arg.strip_prefix("version=") {
            version = value.to_string();
        }
        idx += 1;
    }

    if file_path.is_empty() || api.is_empty() || version.is_empty() {
        println!(
            "❌ CLI Syntax Error.\nRequired usage: clock file=path/file.clk api=OpenGL version=3.3"
        );
        println!("Example: clock file=triangle.clk api=OpenGL ES version=3.0");
        return;
    }

    // Resolve path: expand ~, accept relative and absolute paths, and canonicalize
    fn expand_tilde(p: &str) -> String {
        if let Some(rest) = p.strip_prefix("~/") {
            if let Some(home) = dirs::home_dir() {
                let mut hp = home.to_string_lossy().to_string();
                if !hp.ends_with('/') {
                    hp.push('/');
                }
                return format!("{}{}", hp, rest);
            }
        }
        p.to_string()
    }

    let expanded = expand_tilde(&file_path);
    let mut resolved: Option<PathBuf> = None;

    let candidate = Path::new(&expanded);
    if candidate.exists() {
        resolved = std::fs::canonicalize(candidate).ok();
    } else {
        // Try resolving relative to current working directory explicitly
        if let Ok(cwd) = env::current_dir() {
            let rel = cwd.join(candidate);
            if rel.exists() {
                resolved = std::fs::canonicalize(rel).ok();
            }
        }
    }

    let path = match resolved {
        Some(p) => p,
        None => {
            println!(
                "❌ Input Error: The source file does not exist or could not be resolved: {}",
                file_path
            );
            return;
        }
    };

    let source_code = match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(e) => {
            println!("❌ Critical Error reading source file: {}", e);
            return;
        }
    };

    let mut progress = ProgressBar::new(5);
    progress.step("Reading source");

    let mut lexer = lexer::Lexer::new(&source_code);
    let tokens = lexer.tokenize();
    progress.step("Tokenizing CLKIL");

    let mut parser = parser::Parser::new(tokens);
    let ast_ffi = match parser.parse_to_ffi() {
        Ok(ast) => {
            progress.step("Building AST");
            ast
        }
        Err(err_msg) => {
            println!();
            println!("{}", err_msg);
            return;
        }
    };

    let backend_name = transpiler::backend_display_name(&api, &version);
    progress.step(&format!("Running backend {}/{}", backend_name, version));

    match transpiler::execute_plugin(&api, &version, &ast_ffi) {
        Ok((vertex_stage_source, fragment_stage_source)) => {
            let file_stem = path.file_stem().unwrap().to_string_lossy();
            let parent_dir = path.parent().unwrap_or(Path::new("."));

            let vert_path = parent_dir.join(format!("{}.vert", file_stem));
            let frag_path = parent_dir.join(format!("{}.frag", file_stem));

            fs::write(&vert_path, vertex_stage_source)
                .expect("❌ Critical error writing first pipeline stage");
            fs::write(&frag_path, fragment_stage_source)
                .expect("❌ Critical error writing second pipeline stage");
            progress.finish("Writing generated output");

            println!("✨ [Clock] Transpilation pipeline completed successfully.");
            println!("💾 Pipeline stage 1 exported to: {:?}", vert_path);
            println!("💾 Pipeline stage 2 exported to: {:?}", frag_path);
        }
        Err(e) => {
            println!();
            println!("❌ Backend error in the graphics plugin: {}", e);
        }
    }

    unsafe {
        ast::free_clock_ast(ast_ffi);
    }
}
