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

use crate::ast::*;
use crate::lexer::{SpannedToken, Token};
use std::ffi::CString;
use std::os::raw::c_uint;

pub struct Parser {
    tokens: Vec<SpannedToken>,
    cursor: usize,
}

impl Parser {
    pub fn new(tokens: Vec<SpannedToken>) -> Self {
        Self { tokens, cursor: 0 }
    }

    pub fn parse_to_ffi(&mut self) -> Result<ClockAST, String> {
        let mut structures = Vec::new();
        let mut classes = Vec::new();

        while !self.is_at_end() {
            self.skip_comments();
            let decorators = self.parse_decorators()?;
            self.skip_comments();

            // Allow an optional `public` modifier before top-level declarations
            if self.match_keyword("public") {
                self.skip_comments();
            }

            if self.match_keyword("struct") {
                let s = self.parse_struct(decorators)?;
                structures.push(s);
            } else if self.match_keyword("class") {
                let c = self.parse_class()?;
                classes.push(c);
            } else {
                self.advance();
            }
        }

        if structures.is_empty() {
            return Err(self.error_here("You are required to define at least one 'struct'."));
        }
        if classes.is_empty() {
            return Err(self.error_here("You are required to define at least one 'class'."));
        }

        Ok(ClockAST {
            structures_count: structures.len() as c_uint,
            structures: Box::into_raw(structures.into_boxed_slice()) as *const CStruct,
            classes_count: classes.len() as c_uint,
            classes: Box::into_raw(classes.into_boxed_slice()) as *const CClass,
        })
    }

    fn skip_comments(&mut self) {
        while let Some(SpannedToken {
            token: Token::Comment(_),
            ..
        }) = self.tokens.get(self.cursor)
        {
            self.cursor += 1;
        }
    }

    fn parse_struct(&mut self, _struct_decorators: Vec<Decorator>) -> Result<CStruct, String> {
        let name_token = self.consume_identifier("The name was expected.")?;
        self.consume(Token::OpenBrace, "The opening brace was expected.")?;

        let mut fields = Vec::new();

        while !self.check(Token::CloseBrace) && !self.is_at_end() {
            self.skip_comments();
            if self.check(Token::CloseBrace) {
                break;
            }
            if self.check_keyword("class") || self.check_keyword("struct") {
                return Err(
                    self.error_here("The '}' was expected before starting a new declaration.")
                );
            }

            let field_decorators = self.parse_decorators()?;
            self.skip_comments();

            self.match_keyword("public");
            self.skip_comments();
            if self.check_keyword("class") || self.check_keyword("struct") {
                return Err(
                    self.error_here("The '}' was expected before starting a new declaration.")
                );
            }

            let data_type = self.consume_type_or_identifier("The data type was expected.")?;
            let field_name = self.consume_identifier("The field name was expected.")?;
            self.consume(Token::Semicolon, "The ';' was expected.")?;

            fields.push(CField {
                name: CString::new(field_name).unwrap().into_raw(),
                data_type: CString::new(data_type).unwrap().into_raw(),
                decorators_count: field_decorators.len() as c_uint,
                decorators: Box::into_raw(field_decorators.into_boxed_slice()) as *const Decorator,
            });
        }
        self.consume(Token::CloseBrace, "The '}' was expected.")?;

        Ok(CStruct {
            name: CString::new(name_token).unwrap().into_raw(),
            fields_count: fields.len() as c_uint,
            fields: Box::into_raw(fields.into_boxed_slice()) as *const CField,
        })
    }

    fn parse_class(&mut self) -> Result<CClass, String> {
        let name_token = self.consume_identifier("The name was expected.")?;
        self.consume(Token::OpenBrace, "The '{' was expected.")?;

        let mut methods = Vec::new();

        while !self.check(Token::CloseBrace) && !self.is_at_end() {
            self.skip_comments();
            if self.check(Token::CloseBrace) {
                break;
            }
            if self.check_keyword("class") || self.check_keyword("struct") {
                return Err(
                    self.error_here("The '}' was expected before starting a new declaration.")
                );
            }

            let method_decorators = self.parse_decorators()?;
            self.skip_comments();

            self.match_keyword("public");
            self.skip_comments();
            if self.check_keyword("class") || self.check_keyword("struct") {
                return Err(
                    self.error_here("The '}' was expected before starting a new declaration.")
                );
            }

            if let Ok(return_type) =
                self.consume_type_or_identifier("The return type was expected.")
            {
                self.skip_comments();
                if let Ok(method_name) = self.consume_identifier("The method name was expected.") {
                    self.skip_comments();

                    self.consume_parameter_list()?;

                    let body_raw_text = self.consume_body_block()?;

                    methods.push(CMethod {
                        name: CString::new(method_name).unwrap().into_raw(),
                        return_type: CString::new(return_type).unwrap().into_raw(),
                        decorators_count: method_decorators.len() as c_uint,
                        decorators: Box::into_raw(method_decorators.into_boxed_slice())
                            as *const Decorator,
                        body_raw: CString::new(body_raw_text).unwrap().into_raw(),
                    });
                    continue;
                }
            }
            self.advance();
        }
        self.consume(Token::CloseBrace, "The '}' was expected.")?;

        Ok(CClass {
            name: CString::new(name_token).unwrap().into_raw(),
            methods_count: methods.len() as c_uint,
            methods: Box::into_raw(methods.into_boxed_slice()) as *const CMethod,
        })
    }

    fn parse_decorators(&mut self) -> Result<Vec<Decorator>, String> {
        let mut decorators = Vec::new();
        self.skip_comments();

        while self.match_token(Token::DecoratorStart) {
            let dec_name = self.consume_identifier("The attribute name was expected.")?;
            match dec_name.as_str() {
                "Vertex" => decorators.push(Decorator::Vertex),
                "Fragment" => decorators.push(Decorator::Fragment),
                "Tessellation" => decorators.push(Decorator::Tessellation),
                "Compute" => decorators.push(Decorator::Compute),
                "Builtin" => {
                    self.consume(Token::OpenParenthesis, "The '(' was expected.")?;
                    let param = self.consume_identifier("The value was expected.")?;
                    let builtin = if param == "BuiltinType" {
                        self.consume(Token::Dot, "The '.' was expected.")?;
                        match_builtin_type(&self.consume_identifier("The property was expected.")?)?
                    } else {
                        match_builtin_type(&param)?
                    };
                    self.consume(Token::CloseParenthesis, "The ')' was expected.")?;
                    decorators.push(Decorator::Builtin(builtin));
                }
                "Location" => {
                    self.consume(Token::OpenParenthesis, "The '(' was expected.")?;
                    if let Token::Literal(num) = self.advance().token {
                        decorators.push(Decorator::Location(num.parse::<c_uint>().unwrap_or(0)));
                    } else {
                        return Err(self.error_previous("The location index was expected."));
                    }
                    self.consume(Token::CloseParenthesis, "The ')' was expected.")?;
                }
                _ => {}
            }
            while self.match_token(Token::Comma) {
                self.advance();
            }
            self.consume(Token::DecoratorEnd, "The ']' was expected.")?;

            self.skip_comments();
        }
        Ok(decorators)
    }

    fn consume_body_block(&mut self) -> Result<String, String> {
        self.consume(Token::OpenBrace, "The '{' was expected.")?;
        let mut brace_count = 1;
        let mut raw_body = String::new();

        while brace_count > 0 && !self.is_at_end() {
            let token = self.advance();
            match &token.token {
                Token::OpenBrace => brace_count += 1,
                Token::CloseBrace => brace_count -= 1,
                _ => {}
            }
            if brace_count > 0 {
                raw_body.push_str(&format_token_to_src(&token.token));
            }
        }
        if brace_count > 0 {
            return Err(self.error_here("The closing brace '}' was expected before end of file."));
        }
        Ok(raw_body)
    }

    fn consume_parameter_list(&mut self) -> Result<(), String> {
        self.consume(Token::OpenParenthesis, "The '(' was expected.")?;
        let mut paren_count = 1;

        while paren_count > 0 && !self.is_at_end() {
            let token = self.advance();
            match token.token {
                Token::OpenParenthesis => paren_count += 1,
                Token::CloseParenthesis => paren_count -= 1,
                _ => {}
            }
        }

        if paren_count > 0 {
            return Err(
                self.error_here("The closing parenthesis ')' was expected before end of file.")
            );
        }

        Ok(())
    }

    fn check(&self, token_type: Token) -> bool {
        if self.is_at_end() {
            return false;
        }
        std::mem::discriminant(&self.tokens[self.cursor].token)
            == std::mem::discriminant(&token_type)
    }

    fn advance(&mut self) -> SpannedToken {
        if !self.is_at_end() {
            self.cursor += 1;
        }
        self.tokens[self.cursor - 1].clone()
    }

    fn is_at_end(&self) -> bool {
        self.cursor >= self.tokens.len()
    }

    fn match_token(&mut self, token_type: Token) -> bool {
        if self.check(token_type) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn match_keyword(&mut self, keyword: &str) -> bool {
        if let Some(SpannedToken {
            token: Token::Keyword(k),
            ..
        }) = self.tokens.get(self.cursor)
        {
            if k == keyword {
                self.cursor += 1;
                return true;
            }
        }
        false
    }

    fn check_keyword(&self, keyword: &str) -> bool {
        matches!(
            self.tokens.get(self.cursor),
            Some(SpannedToken { token: Token::Keyword(k), .. }) if k == keyword
        )
    }

    fn consume(&mut self, token_type: Token, msg: &str) -> Result<Token, String> {
        if self.check(token_type.clone()) {
            Ok(self.advance().token)
        } else if token_type == Token::Semicolon {
            Err(self.error_previous(msg))
        } else {
            Err(self.error_here(msg))
        }
    }

    fn consume_identifier(&mut self, msg: &str) -> Result<String, String> {
        if let Some(SpannedToken {
            token: Token::Identifier(id),
            ..
        }) = self.tokens.get(self.cursor)
        {
            let res = id.clone();
            self.cursor += 1;
            Ok(res)
        } else {
            Err(self.error_here(msg))
        }
    }

    fn consume_type_or_identifier(&mut self, msg: &str) -> Result<String, String> {
        match self.tokens.get(self.cursor).map(|spanned| &spanned.token) {
            Some(Token::Identifier(id)) => {
                self.cursor += 1;
                Ok(id.clone())
            }
            Some(Token::Keyword(k)) => {
                self.cursor += 1;
                Ok(k.clone())
            }
            _ => Err(self.error_here(msg)),
        }
    }

    fn error_here(&self, msg: &str) -> String {
        format!(
            "❌ CLKIL Syntax Error at line {}: {}",
            self.current_line(),
            msg
        )
    }

    fn error_previous(&self, msg: &str) -> String {
        format!(
            "❌ CLKIL Syntax Error at line {}: {}",
            self.previous_line(),
            msg
        )
    }

    fn current_line(&self) -> usize {
        self.tokens
            .get(self.cursor)
            .map(|token| token.line)
            .unwrap_or_else(|| self.previous_line())
    }

    fn previous_line(&self) -> usize {
        self.tokens
            .get(self.cursor.saturating_sub(1))
            .map(|token| token.line)
            .unwrap_or(1)
    }
}

fn match_builtin_type(name: &str) -> Result<BuiltinType, String> {
    match name {
        "Position" => Ok(BuiltinType::Position),
        "VertexId" => Ok(BuiltinType::VertexId),
        _ => Err("Built-in unknown".to_string()),
    }
}

fn format_token_to_src(token: &Token) -> String {
    match token {
        Token::Comment(c) => format!("{}\n    ", c),
        Token::Keyword(k) => format!("{} ", k),
        Token::Identifier(i) => format!("{} ", i),
        Token::Literal(l) => format!("{} ", l),
        Token::Semicolon => ";\n    ".to_string(),
        Token::Assign => "= ".to_string(),
        Token::Comma => ", ".to_string(),
        Token::OpenParenthesis => "(".to_string(),
        Token::CloseParenthesis => ") ".to_string(),
        Token::Dot => ".".to_string(),
        Token::DecoratorStart => "[".to_string(),
        Token::DecoratorEnd => "]".to_string(),
        Token::OpenBrace => "{\n        ".to_string(),
        Token::CloseBrace => "}\n    ".to_string(),
    }
}
