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

#![allow(dead_code)]

use std::ffi::CString;
use std::os::raw::{c_char, c_uint};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum BuiltinType {
    Position,
    VertexId,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum Decorator {
    Builtin(BuiltinType),
    Location(c_uint),
    Vertex,
    Fragment,
    Tessellation,
    Compute,
}

#[repr(C)]
pub struct CField {
    pub name: *const c_char,
    pub data_type: *const c_char,
    pub decorators: *const Decorator,
    pub decorators_count: c_uint,
}

#[repr(C)]
pub struct CStruct {
    pub name: *const c_char,
    pub fields: *const CField,
    pub fields_count: c_uint,
}

#[repr(C)]
pub struct CMethod {
    pub name: *const c_char,
    pub return_type: *const c_char,
    pub decorators: *const Decorator,
    pub decorators_count: c_uint,
    pub body_raw: *const c_char,
}

#[repr(C)]
pub struct CClass {
    pub name: *const c_char,
    pub methods: *const CMethod,
    pub methods_count: c_uint,
}

#[repr(C)]
pub struct ClockAST {
    pub structures: *const CStruct,
    pub structures_count: c_uint,
    pub classes: *const CClass,
    pub classes_count: c_uint,
}

#[repr(C)]
pub struct TranspiledPipeline {
    pub vertex_source: *mut c_char,
    pub fragment_source: *mut c_char,
}

/// Frees every allocation owned by a `ClockAST`.
///
/// # Safety
///
/// `ast` must have been produced by Clock's parser using the same allocation
/// layout as this module. All nested pointers and counts must be valid, and
/// this function must be called at most once for a given AST value.
#[no_mangle]
pub unsafe extern "C" fn free_clock_ast(ast: ClockAST) {
    if !ast.structures.is_null() {
        let structs = Vec::from_raw_parts(
            ast.structures as *mut CStruct,
            ast.structures_count as usize,
            ast.structures_count as usize,
        );
        for s in structs {
            if !s.name.is_null() {
                let _ = CString::from_raw(s.name as *mut c_char);
            }
            if !s.fields.is_null() {
                let fields = Vec::from_raw_parts(
                    s.fields as *mut CField,
                    s.fields_count as usize,
                    s.fields_count as usize,
                );
                for f in fields {
                    if !f.name.is_null() {
                        let _ = CString::from_raw(f.name as *mut c_char);
                    }
                    if !f.data_type.is_null() {
                        let _ = CString::from_raw(f.data_type as *mut c_char);
                    }
                    if !f.decorators.is_null() {
                        let _ = Vec::from_raw_parts(
                            f.decorators as *mut Decorator,
                            f.decorators_count as usize,
                            f.decorators_count as usize,
                        );
                    }
                }
            }
        }
    }

    if !ast.classes.is_null() {
        let classes = Vec::from_raw_parts(
            ast.classes as *mut CClass,
            ast.classes_count as usize,
            ast.classes_count as usize,
        );
        for c in classes {
            if !c.name.is_null() {
                let _ = CString::from_raw(c.name as *mut c_char);
            }
            if !c.methods.is_null() {
                let methods = Vec::from_raw_parts(
                    c.methods as *mut CMethod,
                    c.methods_count as usize,
                    c.methods_count as usize,
                );
                for m in methods {
                    if !m.name.is_null() {
                        let _ = CString::from_raw(m.name as *mut c_char);
                    }
                    if !m.return_type.is_null() {
                        let _ = CString::from_raw(m.return_type as *mut c_char);
                    }
                    if !m.body_raw.is_null() {
                        let _ = CString::from_raw(m.body_raw as *mut c_char);
                    }
                    if !m.decorators.is_null() {
                        let _ = Vec::from_raw_parts(
                            m.decorators as *mut Decorator,
                            m.decorators_count as usize,
                            m.decorators_count as usize,
                        );
                    }
                }
            }
        }
    }
}

/// Frees generated pipeline strings returned by a backend plugin.
///
/// # Safety
///
/// Each non-null pointer in `pipeline` must have been allocated with
/// `CString::into_raw` and must not have been freed before. This function must
/// be called at most once for a given pipeline value.
#[no_mangle]
pub unsafe extern "C" fn free_transpiled_pipeline(pipeline: TranspiledPipeline) {
    if !pipeline.vertex_source.is_null() {
        let _ = CString::from_raw(pipeline.vertex_source);
    }
    if !pipeline.fragment_source.is_null() {
        let _ = CString::from_raw(pipeline.fragment_source);
    }
}
