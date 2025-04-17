#![feature(assert_matches)]

use std::assert_matches::assert_matches;

use lexical_analyzer::{
    ModuleTokenStream, OperatorKind::TypeQualifier, PunctuationKind, ReservedKind, Token, TokenKind,
};

#[derive(Debug, Clone)]
pub enum Type {
    Prim(Primitive),
    Struct,
    Enum,
    Function {
        inputs: Option<Vec<(String, Type)>>,
        output: Option<Box<Type>>,
    },
}

impl Type {
    fn push_input(&mut self, parameter_name: String, parameter_type: Type) {
        assert_matches!(self, Type::Function { .. });

        if let Type::Function { inputs, .. } = self {
            if let Some(inputs) = inputs {
                inputs.push((parameter_name, parameter_type));
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum Primitive {
    Bool,
    U8,
    U16,
    U32,
    U64,
    S8,
    S16,
    S32,
    S64,
    F32,
    F64,
}

pub fn eval_ty_from_token(token: Token, token_stream: Option<&mut ModuleTokenStream>) -> Type {
    match token.kind {
        Some(kind) => match kind {
            lexical_analyzer::TokenKind::IntLiteral => eval_int_ty_from_literal(&token),
            lexical_analyzer::TokenKind::FloatLiteral => eval_float_ty_from_literal(&token),
            lexical_analyzer::TokenKind::Identifier => {
                todo!(
                    "Need to implement global symbol table first to resolve identifier: `{identifier:?}`.",
                    identifier = token.content
                )
            }
            lexical_analyzer::TokenKind::Reserved(reserved_kind) => {
                eval_ty_from_reserved_word(reserved_kind, token_stream)
            }
            lexical_analyzer::TokenKind::Punctuation(punctuation_kind) => {
                panic!("Punctuation: `{punctuation_kind:?}` does not have a type to evaluate.")
            }
            lexical_analyzer::TokenKind::Operator(operator_kind) => {
                todo!("Evaluating operators as types not implemented.")
            }
        },
        None => panic!("Got `None` `token.kind`. This is likely an error in the scanner."),
    }
}

fn eval_ty_from_reserved_word(
    reserved_kind: ReservedKind,
    mut token_stream: Option<&mut ModuleTokenStream>,
) -> Type {
    match reserved_kind {
        ReservedKind::Struct => Type::Struct,
        ReservedKind::Enum => Type::Enum,
        ReservedKind::PrimTy(prim_ty) => match prim_ty {
            lexical_analyzer::ScannerPrimKind::Bool => Type::Prim(Primitive::Bool),
            lexical_analyzer::ScannerPrimKind::S8 => Type::Prim(Primitive::S8),
            lexical_analyzer::ScannerPrimKind::S16 => Type::Prim(Primitive::S16),
            lexical_analyzer::ScannerPrimKind::S32 => Type::Prim(Primitive::S32),
            lexical_analyzer::ScannerPrimKind::S64 => Type::Prim(Primitive::S64),
            lexical_analyzer::ScannerPrimKind::U8 => Type::Prim(Primitive::U8),
            lexical_analyzer::ScannerPrimKind::U16 => Type::Prim(Primitive::U16),
            lexical_analyzer::ScannerPrimKind::U32 => Type::Prim(Primitive::U32),
            lexical_analyzer::ScannerPrimKind::U64 => Type::Prim(Primitive::U64),
        },
        ReservedKind::Proc => {
            assert!(
                token_stream.is_some(),
                "Token stream is required to evaluate proc type."
            );
            eval_proc_ty(token_stream.as_mut().unwrap())
        }
        _ => panic!("Reserved word `{reserved_kind:?}` has no type to evaluate."),
    }
}

fn eval_proc_ty(token_stream: &mut ModuleTokenStream<'_>) -> Type {
    let l_parn = token_stream.get_token().unwrap();
    l_parn.assert_kind(TokenKind::Punctuation(PunctuationKind::OpenParen));

    let mut function_type = Type::Function {
        inputs: None,
        output: None,
    };

    let current_token = token_stream.get_token().unwrap();
    while current_token.kind != Some(TokenKind::Punctuation(PunctuationKind::CloseParen)) {
        current_token.assert_kind(TokenKind::Identifier);
        // PERF: Remove clone somehow.
        let parameter_name = String::from(current_token.content.clone());

        let ty_punc = token_stream.get_token().unwrap();
        ty_punc.assert_kind(TokenKind::Operator(TypeQualifier));

        let next_token = token_stream.get_token().unwrap();
        let parameter_type = eval_ty_from_token(next_token, Some(token_stream));

        function_type.push_input(parameter_name, parameter_type);

        // Optional comma. Eat if found.
        if let Some(comma) = token_stream.get_token() {
            comma.assert_kind(TokenKind::Punctuation(PunctuationKind::Comma));
        }
    }

    function_type
}

fn eval_int_ty_from_literal(token: &Token) -> Type {
    let int_literal_postfix = get_postfix_from_num_literal(token);
    match int_literal_postfix {
        Some(postfix) => match postfix {
            NumPostfix::U8 => Type::Prim(Primitive::U8),
            NumPostfix::U16 => Type::Prim(Primitive::U16),
            NumPostfix::U32 => Type::Prim(Primitive::U32),
            NumPostfix::U64 => Type::Prim(Primitive::U64),
            NumPostfix::S8 => Type::Prim(Primitive::S8),
            NumPostfix::S16 => Type::Prim(Primitive::S16),
            NumPostfix::S32 => Type::Prim(Primitive::S32),
            NumPostfix::S64 => Type::Prim(Primitive::S64),
            _ => panic!("Unsupported postfix for int literal: `{}`", token.content),
        },
        None => Type::Prim(Primitive::U32),
    }
}

fn eval_float_ty_from_literal(token: &Token) -> Type {
    let float_literal_postfix = get_postfix_from_num_literal(token);
    match float_literal_postfix {
        Some(postfix) => match postfix {
            NumPostfix::F32 => Type::Prim(Primitive::F32),
            NumPostfix::F64 => Type::Prim(Primitive::F64),
            _ => panic!("Unsupported postfix for float literal: `{}`", token.content),
        },
        None => Type::Prim(Primitive::F32),
    }
}

#[derive(Debug)]
enum NumPostfix {
    U8,
    U16,
    U32,
    U64,
    S8,
    S16,
    S32,
    S64,
    F32,
    F64,
}

fn get_postfix_from_num_literal(token: &Token) -> Option<NumPostfix> {
    // Postfix notation only supported for int and float literals
    assert!(
        matches!(
            token.kind,
            Some(TokenKind::IntLiteral | TokenKind::FloatLiteral),
        ),
        "Expected IntLiteral or FloatLiteral"
    );

    let token_raw = token.content.as_str();

    if token_raw.contains("u8") {
        return Some(NumPostfix::U8);
    } else if token_raw.contains("u16") {
        return Some(NumPostfix::U16);
    } else if token_raw.contains("u32") {
        return Some(NumPostfix::U32);
    } else if token_raw.contains("u64") {
        return Some(NumPostfix::U64);
    } else if token_raw.contains("s8") {
        return Some(NumPostfix::S8);
    } else if token_raw.contains("s16") {
        return Some(NumPostfix::S16);
    } else if token_raw.contains("s32") {
        return Some(NumPostfix::S32);
    } else if token_raw.contains("s64") {
        return Some(NumPostfix::S64);
    } else if token_raw.contains("f32") {
        return Some(NumPostfix::F32);
    } else if token_raw.contains("f64") {
        return Some(NumPostfix::F64);
    } else {
        None
    }
}
