#![feature(core_intrinsics)]

use lexical_analyzer::{
    ModuleTokenStream, OperatorKind, PunctuationKind,
    ReservedKind::{Enum, Proc, Struct},
    Token, TokenKind,
};
use typer::{Type, eval_ty_from_token};

#[derive(Debug)]
pub struct Ast {
    program: Program,
}

impl Ast {
    pub fn new(program: Vec<ModuleTokenStream>) -> Self {
        let program = Program::new(program);
        Self { program }
    }
}

#[derive(Debug)]
struct Program {
    modules: Vec<Module>,
}

impl Program {
    fn new(module_token_streams: Vec<ModuleTokenStream>) -> Self {
        let mut modules: Vec<Module> = Vec::new();
        for module_token_stream in module_token_streams {
            modules.push(Module::new(module_token_stream));
        }

        Self { modules }
    }
}

#[derive(Debug)]
struct Module {
    declarations: Vec<Declaration>,
}

// fn get_token_from_kind(token: Token)

impl Module {
    fn new(mut module: ModuleTokenStream) -> Self {
        let mut declarations = parse_declarations(module, true);
        Self { declarations }
    }
}

fn parse_declarations(mut module_tokens: ModuleTokenStream, parsing: bool) -> Vec<Declaration> {
    let mut declarations: Vec<Declaration> = Vec::new();
    let mut parsing = true;
    while parsing {
        let name = module_tokens
            .get_token()
            .expect("Expected `Identifier` token, found `None`")
            .assert_allowed_kinds(&[
                TokenKind::Identifier,
                TokenKind::Reserved(lexical_analyzer::ReservedKind::Main),
            ]);

        let _ = module_tokens
            .get_token()
            .expect("Expected `TypeQualifier` token, found `None`")
            .assert_kind(TokenKind::Operator(OperatorKind::TypeQualifier));

        // TODO: pull this out into a `parse_type()` function
        // to handle the case of function-types (e.g. (s32, s32) -> s32).
        let ty_tok = module_tokens
            .get_token()
            .expect("Expected `Type` token, found `None`")
            .assert_allowed_kinds(&[
                TokenKind::Reserved(Struct),
                TokenKind::Reserved(Enum),
                TokenKind::Reserved(Proc),
            ]);

        // TODO: Handle function types
        // NOTE: This function eats tokens!
        let ty = typer::eval_ty_from_token(ty_tok, Some(&mut module_tokens));
        let decl_ty = ty.clone();

        let decl_signature = DeclarationSignature::new(name, ty);
        println!("Parsed declaration signature: {decl_signature:?}");

        let _l_brace = module_tokens
            .get_token()
            .expect("Expected `{` token, found `None`")
            .assert_kind(TokenKind::Punctuation(PunctuationKind::OpenBrace));

        let decl_def = parse_declaration_def(&mut module_tokens, decl_ty, parsing);
        println!("Parsed declaration definition: {decl_def:?}");

        let _r_brace = module_tokens
            .get_token()
            .expect("Expected `}` token, found `None`")
            .assert_kind(TokenKind::Punctuation(PunctuationKind::CloseBrace));

        // `Declaration` parsed
        declarations.push(Declaration {
            sig: decl_signature,
            def: decl_def,
        });

        // See if we are done parsing
        if std::intrinsics::unlikely(module_tokens.peek_token().is_none()) {
            parsing = false;
        }
    }

    declarations
}

fn parse_declaration_def(
    module_tokens: &mut ModuleTokenStream<'_>,
    decl_ty: Type,
    parsing: bool,
) -> DeclarationDef {
    match decl_ty {
        Type::Prim(primitive) => todo!(),
        Type::Struct => parse_struct_decl_def(module_tokens),
        Type::Enum => parse_enum_decl_def(module_tokens),
        Type::Function { inputs, output } => todo!(),
    }
}

fn next_token_is(module_tokens: &mut ModuleTokenStream<'_>, token_kind: TokenKind) -> bool {
    let token = module_tokens.peek_token().unwrap();
    let token_kind_found = token.kind.unwrap();
    token_kind == token_kind_found
}

fn consume_next_token(module_tokens: &mut ModuleTokenStream<'_>) {
    // Consume the token
    let _ = module_tokens.get_token().unwrap();
}

fn parse_enum_decl_def(module_tokens: &mut ModuleTokenStream<'_>) -> DeclarationDef {
    use PunctuationKind::{CloseBrace, Comma};
    use TokenKind::{Identifier, Punctuation};

    let mut variants: Vec<Variant> = Vec::new();

    // We have an empty enum if we immedately find a `CloseBrace` token
    if next_token_is(module_tokens, Punctuation(CloseBrace)) {
        consume_next_token(module_tokens);
        return DeclarationDef::Enum { variants };
    }

    // Try to parse enum variants
    while let Some(token) = module_tokens.get_token() {
        token.assert_kind(Identifier);
        let variant = token;
        variants.push(Variant { name: variant });

        // Check for comma
        if next_token_is(module_tokens, Punctuation(Comma)) {
            consume_next_token(module_tokens);
        }

        // Check for closing brace
        let token = module_tokens.peek_token().unwrap();
        let token_kind = token.kind.unwrap();
        if token_kind == Punctuation(CloseBrace) {
            // Note that we do not consume the token here.
            // Instead, we consume it in the `parse_declarations()` function further
            // the call stack.
            break;
        }
    }

    DeclarationDef::Enum { variants }
}

fn parse_struct_decl_def(module_tokens: &mut ModuleTokenStream<'_>) -> DeclarationDef {
    use OperatorKind::TypeQualifier;
    use PunctuationKind::{CloseBrace, Comma};
    use TokenKind::{Identifier, Operator, Punctuation};

    let mut fields: Vec<Field> = Vec::new();

    // We have an empty struct if we immedately find a `CloseBrace` token
    if next_token_is(module_tokens, Punctuation(CloseBrace)) {
        consume_next_token(module_tokens);
        return DeclarationDef::Struct { fields };
    }

    // Struct not empty
    while let Some(token) = module_tokens.get_token() {
        // declaration name
        token.assert_kind(Identifier);
        let name = token;

        // `::`
        let _ = module_tokens
            .get_token()
            .unwrap()
            .assert_kind(Operator(TypeQualifier));

        // Declaration kind
        let ty_tok = module_tokens.get_token().unwrap();

        // Type eval for this declaration kind (i.e. is it a struct or an enum?)
        let ty = eval_ty_from_token(ty_tok, None);
        fields.push(Field::new(name, ty));

        // Check for comma
        if next_token_is(module_tokens, Punctuation(Comma)) {
            consume_next_token(module_tokens);
        }

        // Check for closing brace
        if next_token_is(module_tokens, Punctuation(CloseBrace)) {
            // Note that we do not consume the token here.
            // Instead, we consume it in the `parse_declarations()` function further
            // the call stack.
            break;
        }
    }

    DeclarationDef::Struct { fields }
}

#[derive(Debug)]
struct Declaration {
    sig: DeclarationSignature,
    def: DeclarationDef,
}

#[derive(Debug)]
struct DeclarationSignature {
    name: Token,
    ty: Type,
}

impl DeclarationSignature {
    fn new(name: Token, ty: Type) -> Self {
        Self { name, ty }
    }
}

#[derive(Debug)]
enum DeclarationDef {
    Struct { fields: Vec<Field> },
    Enum { variants: Vec<Variant> },
    Function { def: FunctionDef },
    // Constant,
}

#[derive(Debug)]
struct Variant {
    name: Token,
}

#[derive(Debug)]
struct Field {
    name: Token,
    ty: Type,
}
impl Field {
    fn new(name: Token, ty: Type) -> Self {
        Self { name, ty }
    }
}

#[derive(Debug)]
struct Param {
    name: Token,
    ty: Type,
}

#[derive(Debug)]
struct FunctionDef {
    body: Vec<Statement>,
    expr: Option<Expression>,
}

#[derive(Debug)]
enum Statement {
    Assign,
}

#[derive(Debug)]
enum Expression {
    AddExpr(Box<Expression>, Box<Expression>),
    SubExpr(Box<Expression>, Box<Expression>),
    MulExpr(Box<Expression>, Box<Expression>),
    DivExpr(Box<Expression>, Box<Expression>),
    Atom { inner: Atom },
}

#[derive(Debug)]
enum Atom {
    Literal,
    Identifier,
}
