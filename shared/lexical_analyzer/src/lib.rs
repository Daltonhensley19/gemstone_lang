#![feature(let_chains)]

use module_manager::{Module, ModuleManager};
use span::Span;
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct Token {
    pub content: String,
    pub kind: Option<TokenKind>,
    pub span: Span,
}

impl Token {
    fn new() -> Self {
        Self {
            content: String::new(),
            kind: None,
            span: Span::new(),
        }
    }

    pub fn assert_kind(&self, kind: TokenKind) {
        match self.kind {
            Some(token_kind) => {
                if token_kind == kind {
                    return;
                } else {
                    panic!("Expected token kind: {kind:?}, but got: {token_kind:?}")
                }
            }
            _ => panic!("Expected token kind: {kind:?}, but got: None"),
        }
    }

    pub fn assert_allowed_kinds(self, kinds: &[TokenKind]) -> Self {
        let mut match_found = false;
        let self_kind = self
            .kind
            .expect("Token kind should not be None. Possible error in scanner.");

        for kind in kinds {
            if self_kind == *kind {
                match_found = true;
            }
        }

        match match_found {
            true => self,
            false => panic!("Expected token kinds: {kinds:?}, but got: {self_kind:?}"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ModuleTokenStream<'scanner> {
    pub module: &'scanner Module,
    pub tokens: Vec<Token>,
    pub cursor: usize,
}

impl ModuleTokenStream<'_> {
    pub fn get_token(&mut self) -> Option<Token> {
        let tok = self.tokens.get(self.cursor).cloned();
        self.cursor += 1;
        tok
    }

    pub fn peek_token(&self) -> Option<Token> {
        self.tokens.get(self.cursor).cloned()
    }
}

#[derive(Debug, Clone)]
pub struct Scanner<'scanner> {
    module_manager: &'scanner ModuleManager,
    token_stream: Vec<ModuleTokenStream<'scanner>>,
}

// CTOR
impl<'scanner> Scanner<'scanner> {
    pub fn new(module_manager: &'scanner ModuleManager) -> Self {
        let token_stream = Vec::new();
        Self {
            module_manager,
            token_stream,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReservedKind {
    Struct,
    Enum,
    And,
    Or,
    If,
    Else,
    Main,
    Proc,
    PrimTy(ScannerPrimKind),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScannerPrimKind {
    Bool,
    S8,
    S16,
    S32,
    S64,
    U8,
    U16,
    U32,
    U64,
}

impl TryFrom<&str> for ReservedKind {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "and" => Ok(ReservedKind::And),
            "or" => Ok(ReservedKind::Or),
            "if" => Ok(ReservedKind::If),
            "else" => Ok(ReservedKind::Else),
            "main" => Ok(ReservedKind::Main),
            "struct" => Ok(ReservedKind::Struct),
            "enum" => Ok(ReservedKind::Enum),
            "proc" => Ok(ReservedKind::Proc),
            "bool" => Ok(ReservedKind::PrimTy(ScannerPrimKind::Bool)),
            "s8" => Ok(ReservedKind::PrimTy(ScannerPrimKind::S8)),
            "s16" => Ok(ReservedKind::PrimTy(ScannerPrimKind::S16)),
            "s32" => Ok(ReservedKind::PrimTy(ScannerPrimKind::S32)),
            "s64" => Ok(ReservedKind::PrimTy(ScannerPrimKind::S64)),
            "u8" => Ok(ReservedKind::PrimTy(ScannerPrimKind::U8)),
            "u16" => Ok(ReservedKind::PrimTy(ScannerPrimKind::U16)),
            "u32" => Ok(ReservedKind::PrimTy(ScannerPrimKind::U32)),
            "u64" => Ok(ReservedKind::PrimTy(ScannerPrimKind::U64)),
            _ => Err("Invalid reserved keyword"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PunctuationKind {
    OpenParen,
    CloseParen,
    OpenBracket,
    CloseBracket,
    OpenBrace,
    CloseBrace,
    Comma,
    Dot,
    Semicolon,
}

impl From<char> for PunctuationKind {
    fn from(value: char) -> Self {
        match value {
            '(' => PunctuationKind::OpenParen,
            ')' => PunctuationKind::CloseParen,
            '[' => PunctuationKind::OpenBracket,
            ']' => PunctuationKind::CloseBracket,
            '{' => PunctuationKind::OpenBrace,
            '}' => PunctuationKind::CloseBrace,
            ',' => PunctuationKind::Comma,
            '.' => PunctuationKind::Dot,
            ';' => PunctuationKind::Semicolon,
            _ => panic!("Invalid punctuation char: {}", value),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperatorKind {
    TypeArrow,
    Plus,
    Sub,
    Mul,
    Div,
    TypeQualifier,
    Assign,
    AssignPlus,
    AssignSub,
    AssignMul,
    AssignDiv,
    LT,
    LTE,
    GT,
    Eq,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OperatorComplexity {
    // Single char operator
    Simple,

    // Two char operator
    Complex,
}

impl From<OperatorKind> for OperatorComplexity {
    fn from(value: OperatorKind) -> Self {
        use OperatorComplexity::*;
        match value {
            OperatorKind::TypeArrow => Simple,
            OperatorKind::Plus => Simple,
            OperatorKind::Sub => Simple,
            OperatorKind::Mul => Simple,
            OperatorKind::Div => Simple,
            OperatorKind::TypeQualifier => Complex,
            OperatorKind::Assign => Simple,
            OperatorKind::AssignPlus => Complex,
            OperatorKind::AssignSub => Complex,
            OperatorKind::AssignMul => Complex,
            OperatorKind::AssignDiv => Complex,
            OperatorKind::LT => Simple,
            OperatorKind::LTE => Complex,
            OperatorKind::GT => Simple,
            OperatorKind::Eq => Simple,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    IntLiteral,
    FloatLiteral,
    Identifier,
    Reserved(ReservedKind),
    Punctuation(PunctuationKind),
    Operator(OperatorKind),
}

impl TokenKind {
    #[must_use]
    pub fn is_int_literal(&self) -> bool {
        matches!(self, Self::IntLiteral)
    }

    #[must_use]
    pub fn is_float_literal(&self) -> bool {
        matches!(self, Self::FloatLiteral)
    }
}

impl<'scanner> Scanner<'scanner> {
    pub fn scan(&self) -> Result<Vec<ModuleTokenStream>, ScannerError> {
        let determine_token_kind = |token: &Token, next_char: Option<char>| -> Option<TokenKind> {
            let content = &token.content;
            assert!(content.len() > 0);

            let starts_with = content.chars().nth(0).expect("Expected non-empty string");
            let punctuation = vec!['(', ')', '[', ']', '{', '}', ',', '.', ';'];

            // Identifier or reserved keyword
            if starts_with.is_alphabetic() {
                if let Ok(reserved_kind) = ReservedKind::try_from(content.as_str()) {
                    return Some(TokenKind::Reserved(reserved_kind));
                } else {
                    return Some(TokenKind::Identifier);
                }
            }
            // `IntLiteral` or `FloatLiteral`
            else if starts_with.is_numeric() {
                let has_decimal = content.contains('.');

                if has_decimal {
                    return Some(TokenKind::FloatLiteral);
                } else {
                    return Some(TokenKind::IntLiteral);
                }
            }
            // Punctuation
            else if punctuation.contains(&starts_with) {
                Some(TokenKind::Punctuation(PunctuationKind::from(starts_with)))
            }
            // Operators
            else {
                let next_char = content.chars().nth(1);

                let op_kind = match (starts_with, next_char) {
                    ('<', Some('=')) => Some(OperatorKind::LTE),
                    ('<', _) => Some(OperatorKind::LT),
                    ('>', Some('=')) => Some(OperatorKind::LTE),
                    ('>', _) => Some(OperatorKind::GT),
                    (':', Some(':')) => Some(OperatorKind::TypeQualifier),
                    ('=', Some('=')) => Some(OperatorKind::Eq),
                    ('=', _) => Some(OperatorKind::Assign),
                    ('+', Some('=')) => Some(OperatorKind::AssignPlus),
                    ('-', Some('>')) => Some(OperatorKind::TypeArrow),
                    ('-', Some('=')) => Some(OperatorKind::AssignSub),
                    ('*', Some('=')) => Some(OperatorKind::AssignMul),
                    ('/', Some('=')) => Some(OperatorKind::AssignDiv),
                    ('+', _) => Some(OperatorKind::Plus),
                    ('-', _) => Some(OperatorKind::Sub),
                    ('*', _) => Some(OperatorKind::Mul),
                    ('/', _) => Some(OperatorKind::Div),
                    _ => None,
                };

                if op_kind.is_none() {
                    return None;
                }

                Some(TokenKind::Operator(op_kind.unwrap()))
            }
        };

        let scan_module = |module: &'scanner Module| -> Result<Vec<Token>, ScannerError> {
            let content = &module.src;
            let mut token_stream: Vec<Token> = Vec::new();
            let punctuation = vec!['(', ')', '[', ']', '{', '}', ',', ';'];
            let operator_atoms = vec!['+', '-', '*', '/', '=', '>', '<', ':'];

            let mut token = Token::new();
            let mut cursor = 1;
            let file_size = module.src.len();
            let mut skip_char = false;
            let mut span = Span::new();
            for ch in content.chars() {
                if skip_char {
                    span.incre_col_num();
                    cursor += 1;
                    skip_char = false;
                    continue;
                }

                if ch.is_whitespace() {
                    // `Literal`, `Identifier`, or `Reserved` token created
                    if token.content.len() > 0 {
                        let next_char = content.chars().nth(cursor);
                        let token_kind = determine_token_kind(&token, next_char);
                        token.kind = token_kind;

                        token_stream.push(token.clone());
                        token = Token::new();
                    }
                } else if punctuation.contains(&ch) {
                    // Handle case where valid token touches punctuation
                    // if token.content == "proc" {
                    //     // TODO: replace assert with `gemstone` error handler
                    //     assert!(ch == '(', "Proc keyword should be followed by '('");
                    //
                    //     // Eat `proc` token
                    //     token.kind = Some(TokenKind::Reserved(ReservedKind::Proc));
                    //     token_stream.push(token.clone());
                    //
                    //     token = Token::new();
                    //
                    //     // Remember to eat `(`
                    //     token.content.push(ch);
                    //     token.kind = Some(TokenKind::Punctuation(PunctuationKind::OpenParen));
                    //     token_stream.push(token.clone());
                    //
                    //     token = Token::new();
                    //     continue;
                    // }

                    // Token that touches punctuation
                    if token.content.len() > 0 {
                        let next_char = content.chars().nth(cursor);
                        let token_kind = determine_token_kind(&token, next_char);

                        if token_kind.is_none() {
                            return Err(ScannerError::UnknownTokenTouchingPunctuation);
                        }

                        token.kind = token_kind;
                        token_stream.push(token.clone());
                    }

                    // `Punctuation` token created
                    token = Token::new();
                    token.content.push(ch);
                    token.kind = Some(TokenKind::Punctuation(PunctuationKind::from(ch)));
                    token_stream.push(token.clone());

                    // Onto the next token
                    token = Token::new();
                } else if operator_atoms.contains(&ch) {
                    token.content.push(ch);

                    // Check next_char
                    let next_char = content.chars().nth(cursor);

                    if ch == ':' && next_char != Some(':') {
                        return Err(ScannerError::MalformedTQualifier);
                    }

                    if let Some(next_char) = next_char {
                        match next_char {
                            ':' | '=' | '>' => {
                                skip_char = true;
                                token.content.push(next_char)
                            }
                            _ => (),
                        }
                    }

                    let token_kind = determine_token_kind(&token, next_char);

                    token.kind = token_kind;

                    // `Operator` token created
                    token_stream.push(token.clone());
                    token = Token::new();
                } else {
                    token.content.push(ch);
                }

                cursor += 1;

                // Get EOF token
                if token.content.len() > 0 && cursor > file_size {
                    let next_char = content.chars().nth(cursor + 1);
                    let token_kind = determine_token_kind(&token, next_char);
                    token.kind = token_kind;
                    token_stream.push(token.clone());
                }
            }

            dbg!(&token_stream);

            Ok(token_stream)
        };

        // Create a vec of `ModuleTokenStream`.
        // Each `ModuleTokenStream` contains a `Module` and a `Vec<Token>
        let modules = self.module_manager.get_ref();
        let mut module_token_streams: Vec<ModuleTokenStream> = Vec::new();
        for module in modules.iter() {
            let tokens = scan_module(module)?;
            let module_token_stream = ModuleTokenStream {
                module,
                tokens,
                cursor: 0,
            };

            // Verify module token stream in post-pass
            Scanner::check_module_token_stream(&module_token_stream)?;

            module_token_streams.push(module_token_stream);
        }

        Ok(module_token_streams)
    }
}

#[derive(Debug, Clone, Error)]
pub enum ScannerError {
    #[error("Num literal format not supported.")]
    MalformedNumLit,
    #[error("Type Qualifier operator expects two `:`, but found only one.")]
    MalformedTQualifier,
    #[error("Malformed token. Cannot determine token touching punctuation.")]
    UnknownTokenTouchingPunctuation,
}

impl<'scanner> Scanner<'scanner> {
    fn check_module_token_stream(tok_stream: &ModuleTokenStream) -> Result<(), ScannerError> {
        Scanner::assert_no_none_token_kinds(tok_stream)?;
        Scanner::check_num_tokens(tok_stream)
    }

    fn assert_no_none_token_kinds(tok_stream: &ModuleTokenStream) -> Result<(), ScannerError> {
        for tok in tok_stream.tokens.iter() {
            assert!(
                tok.kind.is_some(),
                "token kind should not be none at this point. Bug in scanner. `{tok:?}`"
            );
        }

        Ok(())
    }

    fn check_num_tokens(tok_stream: &ModuleTokenStream) -> Result<(), ScannerError> {
        let num_tokens = tok_stream
            .tokens
            .iter()
            .filter(|t| t.kind.unwrap().is_int_literal() || t.kind.unwrap().is_float_literal());

        // Helper to verify int literal format
        let check_int_literal = |int_literal: &Token| {
            for ch in int_literal.content.chars() {
                // TODO: Add support for int literals with special
                // formatting postfixes (e.g. `2i8`,
                // `34u32`).
                if !ch.is_numeric() {
                    return Err(ScannerError::MalformedNumLit);
                }
            }

            Ok(())
        };

        // Helper to verify float literal format
        let check_float_literal = |float_literal: &Token| {
            let decimal_idx = float_literal
                .content
                .chars()
                .position(|c| c == '.')
                .expect("Expected decimal.");

            for ch in float_literal.content[decimal_idx..].chars() {
                if !ch.is_numeric() {
                    return Err(ScannerError::MalformedNumLit);
                }
            }

            Ok(())
        };

        for num_tok in num_tokens {
            let kind = num_tok.kind.unwrap();
            if let TokenKind::IntLiteral = kind {
                check_int_literal(num_tok)?;
            }

            if let TokenKind::FloatLiteral = kind {
                check_float_literal(num_tok)?;
            }
        }

        Ok(())
    }
}
