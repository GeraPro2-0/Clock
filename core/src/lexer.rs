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

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Keyword(String),
    Identifier(String),
    Literal(String),
    Comment(String),
    DecoratorStart,   // [
    DecoratorEnd,     // ]
    OpenBrace,        // {
    CloseBrace,       // }
    OpenParenthesis,  // (
    CloseParenthesis, // )
    Dot,              // .
    Semicolon,        // ;
    Assign,           // =
    Comma,            // ,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpannedToken {
    pub token: Token,
    pub line: usize,
}

pub struct Lexer<'a> {
    source: &'a str,
    chars: Vec<char>,
    cursor: usize,
    line: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            chars: source.chars().collect(),
            cursor: 0,
            line: 1,
        }
    }

    pub fn tokenize(&mut self) -> Vec<SpannedToken> {
        let mut tokens = Vec::new();

        while self.cursor < self.chars.len() {
            let ch = self.chars[self.cursor];

            // Ignore blank spaces
            if ch.is_whitespace() {
                if ch == '\n' {
                    self.line += 1;
                }
                self.cursor += 1;
                continue;
            }

            // Comment Management
            if ch == '/' && self.peek() == Some('/') {
                let start_comment = self.cursor;
                let start_line = self.line;
                while self.cursor < self.chars.len() && self.chars[self.cursor] != '\n' {
                    self.cursor += 1;
                }
                // Capture the complete line comment (including the //)
                let comment_str: String = self.chars[start_comment..self.cursor].iter().collect();
                tokens.push(self.spanned_at(Token::Comment(comment_str), start_line));
                continue;
            }

            if ch == '/' && self.peek() == Some('*') {
                let start_comment = self.cursor;
                let start_line = self.line;
                self.cursor += 2; // Skip the slash and the asterisk
                while self.cursor < self.chars.len()
                    && !(self.chars[self.cursor] == '*' && self.peek() == Some('/'))
                {
                    if self.chars[self.cursor] == '\n' {
                        self.line += 1;
                    }
                    self.cursor += 1;
                }
                if self.cursor < self.chars.len() {
                    self.cursor += 2;
                } // Close the block

                let comment_str: String = self.chars[start_comment..self.cursor].iter().collect();
                tokens.push(self.spanned_at(Token::Comment(comment_str), start_line));
                continue;
            }

            // SSymbols and Operators
            match ch {
                '[' => {
                    tokens.push(self.spanned(Token::DecoratorStart));
                    self.cursor += 1;
                    continue;
                }
                ']' => {
                    tokens.push(self.spanned(Token::DecoratorEnd));
                    self.cursor += 1;
                    continue;
                }
                '{' => {
                    tokens.push(self.spanned(Token::OpenBrace));
                    self.cursor += 1;
                    continue;
                }
                '}' => {
                    tokens.push(self.spanned(Token::CloseBrace));
                    self.cursor += 1;
                    continue;
                }
                '(' => {
                    tokens.push(self.spanned(Token::OpenParenthesis));
                    self.cursor += 1;
                    continue;
                }
                ')' => {
                    tokens.push(self.spanned(Token::CloseParenthesis));
                    self.cursor += 1;
                    continue;
                }
                '.' if self.peek().is_some_and(|next| !next.is_numeric()) => {
                    tokens.push(self.spanned(Token::Dot));
                    self.cursor += 1;
                    continue;
                }
                ';' => {
                    tokens.push(self.spanned(Token::Semicolon));
                    self.cursor += 1;
                    continue;
                }
                '=' => {
                    tokens.push(self.spanned(Token::Assign));
                    self.cursor += 1;
                    continue;
                }
                ',' => {
                    tokens.push(self.spanned(Token::Comma));
                    self.cursor += 1;
                    continue;
                }
                _ => {}
            }

            // Identifiers and Reserved Words
            if ch.is_alphabetic() || ch == '_' {
                let mut ident = String::new();
                while self.cursor < self.chars.len()
                    && (self.chars[self.cursor].is_alphanumeric() || self.chars[self.cursor] == '_')
                {
                    ident.push(self.chars[self.cursor]);
                    self.cursor += 1;
                }

                match ident.as_str() {
                    "public" | "struct" | "class" | "return" | "new" | "uint" => {
                        tokens.push(self.spanned(Token::Keyword(ident)));
                    }
                    _ => tokens.push(self.spanned(Token::Identifier(ident))),
                }
                continue;
            }

            // Basic Numerical Literals
            if ch == '-' && matches!(self.peek(), Some(next) if next.is_numeric() || next == '.')
                || ch.is_numeric()
                || ch == '.'
            {
                let mut lit = String::new();
                lit.push(ch);
                self.cursor += 1;

                while self.cursor < self.chars.len()
                    && (self.chars[self.cursor].is_numeric()
                        || self.chars[self.cursor] == '.'
                        || self.chars[self.cursor] == 'f')
                {
                    lit.push(self.chars[self.cursor]);
                    self.cursor += 1;
                }
                tokens.push(self.spanned(Token::Literal(lit)));
                continue;
            }

            self.cursor += 1;
        }

        tokens
    }

    fn peek(&self) -> Option<char> {
        if self.cursor + 1 < self.chars.len() {
            Some(self.chars[self.cursor + 1])
        } else {
            None
        }
    }

    fn spanned(&self, token: Token) -> SpannedToken {
        self.spanned_at(token, self.line)
    }

    fn spanned_at(&self, token: Token, line: usize) -> SpannedToken {
        SpannedToken { token, line }
    }
}
