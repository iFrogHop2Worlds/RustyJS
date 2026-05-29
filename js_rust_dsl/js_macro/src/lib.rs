extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::{Ident, Lit, LitStr, Token, braced, parenthesized, parse_macro_input};

pub(crate) mod parser {
    use super::*;
    use syn::token::{Brace, Bracket, Paren};
    use syn::{bracketed, custom_keyword};

    custom_keyword!(_const);
    custom_keyword!(function);
    custom_keyword!(this);
    custom_keyword!(catch);
    custom_keyword!(finally);
    custom_keyword!(throw);

    pub struct JsScript {
        pub(crate) statements: Vec<JsStatement>,
    }

    #[derive(Clone, Debug)]
    pub enum JsStatement {
        LetDecl(Ident, JsExpression),
        ConstDecl(Ident, JsExpression),
        IfStatement(JsExpression, JsBlock, Option<JsBlock>),
        TryCatchFinally {
            try_block: JsBlock,
            catch_param: Option<Ident>,
            catch_block: Option<JsBlock>,
            finally_block: Option<JsBlock>,
        },
        Throw(JsExpression),
        FunctionDecl(JsFunctionDecl),
        ForLoop(JsForLoop),
        WhileLoop(JsWhileLoop),
        DoWhileLoop(JsDoWhileLoop),
        Expression(JsExpression),
        Return(Option<JsExpression>),
    }

    #[derive(Clone, Debug)]
    pub enum JsExpression {
        Literal(JsLiteral),
        Identifier(Ident),
        This,
        BinaryOp(Box<JsExpression>, JsBinaryOp, Box<JsExpression>),
        UnaryOp(JsUnaryOp, Box<JsExpression>),
        LogicalOp(Box<JsExpression>, JsLogicalOp, Box<JsExpression>),
        Assignment(Ident, Box<JsExpression>),
        MemberAssignment(Box<JsExpression>, Ident, Box<JsExpression>),
        IndexAssignment(Box<JsExpression>, Box<JsExpression>, Box<JsExpression>),
        Call(Ident, Vec<JsExpression>),
        Array(Vec<JsExpression>),
        Object(JsObject),
        MemberAccess(Box<JsExpression>, Ident),
        IndexAccess(Box<JsExpression>, Box<JsExpression>),
        MethodCall {
            object: Box<JsExpression>,
            method: Ident,
            args: Vec<JsExpression>,
        },
    }

    #[derive(Clone)]
    pub struct JsObject {
        pub(crate) properties: Punctuated<JsProperty, Token![,]>,
    }

    impl std::fmt::Debug for JsObject {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("JsObject")
                .field("properties", &self.properties.iter().collect::<Vec<_>>())
                .finish()
        }
    }

    #[derive(Clone)]
    pub struct JsProperty {
        pub(crate) key: String,
        pub(crate) value: JsExpression,
    }

    impl std::fmt::Debug for JsProperty {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("JsProperty")
                .field("key", &self.key)
                .field("value", &self.value)
                .finish()
        }
    }

    #[derive(Clone, Debug)]
    pub enum JsBinaryOp {
        Add,
        Subtract,
        Multiply,
        Divide,
        Remainder,
        Equal,
        NotEqual,
        StrictEqual,
        StrictNotEqual,
        LessThan,
        GreaterThan,
        LessThanOrEqual,
        GreaterThanOrEqual,
    }

    #[derive(Clone, Debug)]
    pub enum JsUnaryOp {
        Negate,
        Not,
    }

    #[derive(Clone, Debug)]
    pub enum JsLogicalOp {
        And,
        Or,
    }

    #[derive(Clone, Debug)]
    pub enum JsLiteral {
        Number(f64),
        String(String),
        Boolean(bool),
        Null,
        Undefined,
        Function(JsFunctionDecl),
    }

    #[derive(Clone, Debug)]
    pub struct JsBlock {
        pub(crate) statements: Vec<JsStatement>,
    }

    #[derive(Clone, Debug)]
    pub struct JsFunctionDecl {
        pub(crate) name: Option<Ident>,
        pub(crate) params: Vec<Ident>,
        pub(crate) body: JsBlock,
    }

    #[derive(Clone, Debug)]
    pub struct JsWhileLoop {
        pub(crate) condition: JsExpression,
        pub(crate) body: JsBlock,
    }

    #[derive(Clone, Debug)]
    pub struct JsDoWhileLoop {
        pub(crate) body: JsBlock,
        pub(crate) condition: JsExpression,
    }

    #[derive(Clone, Debug)]
    pub struct JsForLoop {
        pub(crate) init: Option<Box<JsStatement>>,
        pub(crate) condition: Option<JsExpression>,
        pub(crate) increment: Option<Box<JsExpression>>,
        pub(crate) body: JsBlock,
    }

    impl Parse for JsScript {
        fn parse(input: ParseStream) -> Result<Self> {
            let mut statements = Vec::new();
            while !input.is_empty() {
                statements.push(input.parse()?);
            }
            Ok(JsScript { statements })
        }
    }

    impl Parse for JsStatement {
        fn parse(input: ParseStream) -> Result<Self> {
            let lookahead = input.lookahead1();
            if lookahead.peek(Token![let])
                || lookahead.peek(Token![const])
                || lookahead.peek(self::_const)
            {
                let is_const = if input.peek(Token![let]) {
                    input.parse::<Token![let]>()?;
                    false
                } else if input.peek(Token![const]) {
                    input.parse::<Token![const]>()?;
                    true
                } else {
                    input.parse::<self::_const>()?;
                    true
                };
                let id: Ident = input.parse()?;
                input.parse::<Token![=]>()?;
                let expr: JsExpression = input.parse()?;
                input.parse::<Token![;]>()?;
                if is_const {
                    Ok(JsStatement::ConstDecl(id, expr))
                } else {
                    Ok(JsStatement::LetDecl(id, expr))
                }
            } else if lookahead.peek(Token![if]) {
                input.parse::<Token![if]>()?;
                let content;
                parenthesized!(content in input);
                let cond = content.parse()?;
                let body = input.parse()?;
                let else_body = if input.peek(Token![else]) {
                    input.parse::<Token![else]>()?;
                    if input.peek(Token![if]) {
                        Some(JsBlock {
                            statements: vec![input.parse()?],
                        })
                    } else {
                        Some(input.parse()?)
                    }
                } else {
                    None
                };
                Ok(JsStatement::IfStatement(cond, body, else_body))
            } else if lookahead.peek(Token![try]) {
                input.parse::<Token![try]>()?;
                let try_block: JsBlock = input.parse()?;

                let (catch_param, catch_block) = if input.peek(self::catch) {
                    input.parse::<self::catch>()?;
                    let param = if input.peek(Paren) {
                        let content;
                        parenthesized!(content in input);
                        if content.is_empty() {
                            None
                        } else {
                            Some(content.parse::<Ident>()?)
                        }
                    } else {
                        None
                    };
                    let block: JsBlock = input.parse()?;
                    (param, Some(block))
                } else {
                    (None, None)
                };

                let finally_block = if input.peek(self::finally) {
                    input.parse::<self::finally>()?;
                    Some(input.parse::<JsBlock>()?)
                } else {
                    None
                };

                if catch_block.is_none() && finally_block.is_none() {
                    return Err(input.error("try requires catch and/or finally"));
                }

                Ok(JsStatement::TryCatchFinally {
                    try_block,
                    catch_param,
                    catch_block,
                    finally_block,
                })
            } else if lookahead.peek(self::throw) {
                input.parse::<self::throw>()?;
                let expr: JsExpression = input.parse()?;
                input.parse::<Token![;]>()?;
                Ok(JsStatement::Throw(expr))
            } else if lookahead.peek(function) {
                Ok(JsStatement::FunctionDecl(input.parse()?))
            } else if lookahead.peek(Token![for]) {
                Ok(JsStatement::ForLoop(input.parse()?))
            } else if lookahead.peek(Token![while]) {
                Ok(JsStatement::WhileLoop(input.parse()?))
            } else if lookahead.peek(Token![do]) {
                Ok(JsStatement::DoWhileLoop(input.parse()?))
            } else if input.peek(Token![return]) {
                input.parse::<Token![return]>()?;
                let expr = if input.peek(Token![;]) {
                    None
                } else {
                    Some(input.parse::<JsExpression>()?)
                };
                input.parse::<Token![;]>()?;
                Ok(JsStatement::Return(expr))
            } else {
                let expr = input.parse()?;
                input.parse::<Token![;]>()?;
                Ok(JsStatement::Expression(expr))
            }
        }
    }

    fn parse_postfix_expression(input: ParseStream) -> Result<JsExpression> {
        let mut expr = parse_primary_expression(input)?;

        loop {
            if input.peek(Token![.]) {
                input.parse::<Token![.]>()?;
                let member: Ident = input.parse()?;

                if input.peek(Paren) {
                    // Method call
                    let content;
                    parenthesized!(content in input);
                    let args = if content.is_empty() {
                        Vec::new()
                    } else {
                        Punctuated::<JsExpression, Token![,]>::parse_terminated(&content)?
                            .into_iter()
                            .collect()
                    };

                    expr = JsExpression::MethodCall {
                        object: Box::new(expr),
                        method: member,
                        args,
                    };
                } else {
                    // Member access
                    expr = JsExpression::MemberAccess(Box::new(expr), member);
                }
            } else if input.peek(Bracket) {
                let content;
                bracketed!(content in input);
                let index: JsExpression = content.parse()?;
                expr = JsExpression::IndexAccess(Box::new(expr), Box::new(index));
            } else if peek_double_plus(input) || peek_double_minus(input) {
                let op = if peek_double_plus(input) {
                    input.parse::<Token![+]>()?;
                    input.parse::<Token![+]>()?;
                    JsBinaryOp::Add
                } else {
                    input.parse::<Token![-]>()?;
                    input.parse::<Token![-]>()?;
                    JsBinaryOp::Subtract
                };
                let one = JsExpression::Literal(JsLiteral::Number(1.0));
                expr = match expr {
                    JsExpression::Identifier(ident) => JsExpression::Assignment(
                        ident.clone(),
                        Box::new(JsExpression::BinaryOp(
                            Box::new(JsExpression::Identifier(ident)),
                            op,
                            Box::new(one),
                        )),
                    ),
                    JsExpression::MemberAccess(obj, prop) => JsExpression::MemberAssignment(
                        obj.clone(),
                        prop.clone(),
                        Box::new(JsExpression::BinaryOp(
                            Box::new(JsExpression::MemberAccess(obj, prop)),
                            op,
                            Box::new(one),
                        )),
                    ),
                    JsExpression::IndexAccess(obj, index) => JsExpression::IndexAssignment(
                        obj.clone(),
                        index.clone(),
                        Box::new(JsExpression::BinaryOp(
                            Box::new(JsExpression::IndexAccess(obj, index)),
                            op,
                            Box::new(one),
                        )),
                    ),
                    _ => return Err(syn::Error::new(input.span(), "invalid update target")),
                };
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn peek_double_plus(input: ParseStream) -> bool {
        let fork = input.fork();
        fork.parse::<Token![+]>().is_ok() && fork.parse::<Token![+]>().is_ok()
    }

    fn peek_double_minus(input: ParseStream) -> bool {
        let fork = input.fork();
        fork.parse::<Token![-]>().is_ok() && fork.parse::<Token![-]>().is_ok()
    }

    fn parse_primary_expression(input: ParseStream) -> Result<JsExpression> {
        if input.peek(Paren) {
            let content;
            parenthesized!(content in input);
            return content.parse();
        }

        if input.peek(this) {
            input.parse::<this>()?;
            return Ok(JsExpression::This);
        }

        if input.peek(function) {
            let func: JsFunctionDecl = input.parse()?;
            return Ok(JsExpression::Literal(JsLiteral::Function(func)));
        }

        if input.peek(Bracket) {
            let content;
            bracketed!(content in input);
            let elements = if content.is_empty() {
                Vec::new()
            } else {
                Punctuated::<JsExpression, Token![,]>::parse_terminated(&content)?
                    .into_iter()
                    .collect()
            };
            return Ok(JsExpression::Array(elements));
        }

        if input.peek(Brace) {
            let obj: JsObject = input.parse()?;
            return Ok(JsExpression::Object(obj));
        }

        if let Ok(lit) = input.parse::<Lit>() {
            return Ok(JsExpression::Literal(JsLiteral::try_from(lit)?));
        }

        if input.peek(Ident) {
            let ident: Ident = input.parse()?;

            return if input.peek(Paren) {
                // Function call
                let content;
                parenthesized!(content in input);
                let args = if content.is_empty() {
                    Vec::new()
                } else {
                    Punctuated::<JsExpression, Token![,]>::parse_terminated(&content)?
                        .into_iter()
                        .collect()
                };
                Ok(JsExpression::Call(ident, args))
            } else {
                if ident == "this" {
                    Ok(JsExpression::This)
                } else if ident == "null" {
                    Ok(JsExpression::Literal(JsLiteral::Null))
                } else if ident == "undefined" {
                    Ok(JsExpression::Literal(JsLiteral::Undefined))
                } else {
                    Ok(JsExpression::Identifier(ident))
                }
                //Ok(JsExpression::Identifier(ident))
            };
        }

        Err(input.error("expected expression"))
    }

    fn parse_unary_expression(input: ParseStream) -> Result<JsExpression> {
        if input.peek(Token![-]) {
            input.parse::<Token![-]>()?;
            let expr = parse_unary_expression(input)?;
            return Ok(JsExpression::UnaryOp(JsUnaryOp::Negate, Box::new(expr)));
        }
        if input.peek(Token![!]) {
            input.parse::<Token![!]>()?;
            let expr = parse_unary_expression(input)?;
            return Ok(JsExpression::UnaryOp(JsUnaryOp::Not, Box::new(expr)));
        }
        parse_postfix_expression(input)
    }

    fn parse_multiplicative_expression(input: ParseStream) -> Result<JsExpression> {
        let mut expr = parse_unary_expression(input)?;

        while (input.peek(Token![*]) && !input.peek(Token![*=]))
            || (input.peek(Token![/]) && !input.peek(Token![/=]))
            || (input.peek(Token![%]) && !input.peek(Token![%=]))
        {
            let op = if input.peek(Token![*]) {
                input.parse::<Token![*]>()?;
                JsBinaryOp::Multiply
            } else if input.peek(Token![/]) {
                input.parse::<Token![/]>()?;
                JsBinaryOp::Divide
            } else {
                input.parse::<Token![%]>()?;
                JsBinaryOp::Remainder
            };
            let right = parse_unary_expression(input)?;
            expr = JsExpression::BinaryOp(Box::new(expr), op, Box::new(right));
        }

        Ok(expr)
    }

    fn parse_additive_expression(input: ParseStream) -> Result<JsExpression> {
        let mut expr = parse_multiplicative_expression(input)?; // Change this line

        while (input.peek(Token![+]) && !input.peek(Token![+=]))
            || (input.peek(Token![-]) && !input.peek(Token![-=]))
        {
            if input.peek(Token![+]) {
                input.parse::<Token![+]>()?;
                let right = parse_multiplicative_expression(input)?; // Change this line
                expr = JsExpression::BinaryOp(Box::new(expr), JsBinaryOp::Add, Box::new(right));
            } else {
                input.parse::<Token![-]>()?;
                let right = parse_multiplicative_expression(input)?; // Change this line
                expr =
                    JsExpression::BinaryOp(Box::new(expr), JsBinaryOp::Subtract, Box::new(right));
            }
        }

        Ok(expr)
    }

    fn parse_comparison_expression(input: ParseStream) -> Result<JsExpression> {
        let mut expr = parse_additive_expression(input)?;
        while input.peek(Token![==])
            || input.peek(Token![!=])
            || input.peek(Token![<])
            || input.peek(Token![>])
            || input.peek(Token![<=])
            || input.peek(Token![>=])
        {
            let op = if input.peek(Token![==]) {
                input.parse::<Token![==]>()?;
                if input.peek(Token![=]) {
                    input.parse::<Token![=]>()?;
                    JsBinaryOp::StrictEqual
                } else {
                    JsBinaryOp::Equal
                }
            } else if input.peek(Token![!=]) {
                input.parse::<Token![!=]>()?;
                if input.peek(Token![=]) {
                    input.parse::<Token![=]>()?;
                    JsBinaryOp::StrictNotEqual
                } else {
                    JsBinaryOp::NotEqual
                }
            } else if input.peek(Token![<=]) {
                input.parse::<Token![<=]>()?;
                JsBinaryOp::LessThanOrEqual
            } else if input.peek(Token![>=]) {
                input.parse::<Token![>=]>()?;
                JsBinaryOp::GreaterThanOrEqual
            } else if input.peek(Token![<]) {
                input.parse::<Token![<]>()?;
                JsBinaryOp::LessThan
            } else if input.peek(Token![>]) {
                input.parse::<Token![>]>()?;
                JsBinaryOp::GreaterThan
            } else {
                return Err(syn::Error::new(
                    input.span(),
                    "Unsupported comparison operator",
                ));
            };
            let right = parse_additive_expression(input)?;
            expr = JsExpression::BinaryOp(Box::new(expr), op, Box::new(right));
        }
        Ok(expr)
    }

    fn parse_logical_and_expression(input: ParseStream) -> Result<JsExpression> {
        let mut expr = parse_comparison_expression(input)?;
        while input.peek(Token![&&]) {
            input.parse::<Token![&&]>()?;
            let right = parse_comparison_expression(input)?;
            expr = JsExpression::LogicalOp(Box::new(expr), JsLogicalOp::And, Box::new(right));
        }
        Ok(expr)
    }

    fn parse_logical_or_expression(input: ParseStream) -> Result<JsExpression> {
        let mut expr = parse_logical_and_expression(input)?;
        while input.peek(Token![||]) {
            input.parse::<Token![||]>()?;
            let right = parse_logical_and_expression(input)?;
            expr = JsExpression::LogicalOp(Box::new(expr), JsLogicalOp::Or, Box::new(right));
        }
        Ok(expr)
    }

    fn parse_assignment_expression(input: ParseStream) -> Result<JsExpression> {
        let expr = parse_logical_or_expression(input)?;

        if input.peek(Token![=]) && !input.peek(Token![==]) {
            input.parse::<Token![=]>()?;
            let value = parse_assignment_expression(input)?;

            match &expr {
                // Use a reference here
                JsExpression::Identifier(ident) => {
                    return Ok(JsExpression::Assignment(ident.clone(), Box::new(value)));
                }
                JsExpression::MemberAccess(obj, prop) => {
                    return Ok(JsExpression::MemberAssignment(
                        obj.clone(),
                        prop.clone(),
                        Box::new(value),
                    ));
                }
                JsExpression::IndexAccess(obj, index) => {
                    return Ok(JsExpression::IndexAssignment(
                        obj.clone(),
                        index.clone(),
                        Box::new(value),
                    ));
                }
                _ => {
                    // For any other expression type, we can't do assignment, fall through
                }
            }
        } else if input.peek(Token![+=])
            || input.peek(Token![-=])
            || input.peek(Token![*=])
            || input.peek(Token![/=])
            || input.peek(Token![%=])
        {
            let op = if input.peek(Token![+=]) {
                input.parse::<Token![+=]>()?;
                JsBinaryOp::Add
            } else if input.peek(Token![-=]) {
                input.parse::<Token![-=]>()?;
                JsBinaryOp::Subtract
            } else if input.peek(Token![*=]) {
                input.parse::<Token![*=]>()?;
                JsBinaryOp::Multiply
            } else if input.peek(Token![/=]) {
                input.parse::<Token![/=]>()?;
                JsBinaryOp::Divide
            } else {
                input.parse::<Token![%=]>()?;
                JsBinaryOp::Remainder
            };
            let value = parse_assignment_expression(input)?;

            match &expr {
                JsExpression::Identifier(ident) => {
                    return Ok(JsExpression::Assignment(
                        ident.clone(),
                        Box::new(JsExpression::BinaryOp(
                            Box::new(JsExpression::Identifier(ident.clone())),
                            op,
                            Box::new(value),
                        )),
                    ));
                }
                JsExpression::MemberAccess(obj, prop) => {
                    return Ok(JsExpression::MemberAssignment(
                        obj.clone(),
                        prop.clone(),
                        Box::new(JsExpression::BinaryOp(
                            Box::new(JsExpression::MemberAccess(obj.clone(), prop.clone())),
                            op,
                            Box::new(value),
                        )),
                    ));
                }
                JsExpression::IndexAccess(obj, index) => {
                    return Ok(JsExpression::IndexAssignment(
                        obj.clone(),
                        index.clone(),
                        Box::new(JsExpression::BinaryOp(
                            Box::new(JsExpression::IndexAccess(obj.clone(), index.clone())),
                            op,
                            Box::new(value),
                        )),
                    ));
                }
                _ => {}
            }
        }

        Ok(expr) // expr is still available since we only borrowed it above
    }

    impl Parse for JsExpression {
        fn parse(input: ParseStream) -> Result<Self> {
            parse_assignment_expression(input)
        }
    }

    impl TryFrom<Lit> for JsLiteral {
        type Error = syn::Error;
        fn try_from(lit: Lit) -> Result<Self> {
            match lit {
                Lit::Str(s) => Ok(JsLiteral::String(s.value())),
                Lit::Int(i) => Ok(JsLiteral::Number(i.base10_parse()?)),
                Lit::Float(f) => Ok(JsLiteral::Number(f.base10_parse()?)),
                Lit::Bool(b) => Ok(JsLiteral::Boolean(b.value)),
                _ => Err(syn::Error::new_spanned(lit, "Unsupported literal type")),
            }
        }
    }

    impl Parse for JsBlock {
        fn parse(input: ParseStream) -> Result<Self> {
            let content;
            braced!(content in input);
            let mut statements = Vec::new();
            while !content.is_empty() {
                statements.push(content.parse()?);
            }
            Ok(JsBlock { statements })
        }
    }

    impl Parse for JsFunctionDecl {
        fn parse(input: ParseStream) -> Result<Self> {
            input.parse::<self::function>()?;
            let name = if input.peek(Ident) {
                Some(input.parse()?)
            } else {
                None
            };

            let content;
            parenthesized!(content in input);
            let params_parser = Punctuated::<Ident, Token![,]>::parse_terminated;
            let params = params_parser(&content)?;

            let body = input.parse()?;

            Ok(JsFunctionDecl {
                name,
                params: params.into_iter().collect(),
                body,
            })
        }
    }

    impl Parse for JsWhileLoop {
        fn parse(input: ParseStream) -> Result<Self> {
            input.parse::<Token![while]>()?;
            let content;
            parenthesized!(content in input);
            let condition = content.parse()?;
            let body = input.parse()?;
            Ok(JsWhileLoop { condition, body })
        }
    }

    impl Parse for JsDoWhileLoop {
        fn parse(input: ParseStream) -> Result<Self> {
            input.parse::<Token![do]>()?;
            let body = input.parse()?;
            input.parse::<Token![while]>()?;
            let content;
            parenthesized!(content in input);
            let condition = content.parse()?;
            input.parse::<Token![;]>()?;
            Ok(JsDoWhileLoop { body, condition })
        }
    }

    impl Parse for JsForLoop {
        fn parse(input: ParseStream) -> Result<Self> {
            input.parse::<Token![for]>()?;
            let content;
            parenthesized!(content in input);

            let init = if content.peek(Token![;]) {
                None
            } else {
                Some(Box::new(content.parse::<JsStatement>()?))
            };
            if !content.peek(Token![;]) {
                if let Some(ref s) = init {
                    if let JsStatement::Expression(_) = **s {
                        content.parse::<Token![;]>()?;
                    }
                }
            } else {
                content.parse::<Token![;]>()?;
            }

            let condition = if content.peek(Token![;]) {
                None
            } else {
                Some(content.parse()?)
            };
            content.parse::<Token![;]>()?;

            let increment = if content.is_empty() {
                None
            } else {
                Some(Box::new(content.parse()?))
            };

            let body = input.parse()?;

            Ok(JsForLoop {
                init,
                condition,
                increment,
                body,
            })
        }
    }

    impl Parse for JsObject {
        fn parse(input: ParseStream) -> Result<Self> {
            let content;
            braced!(content in input);
            Ok(JsObject {
                properties: content.parse_terminated(JsProperty::parse, Token![,])?,
            })
        }
    }

    impl Parse for JsProperty {
        fn parse(input: ParseStream) -> Result<Self> {
            let (key, shorthand_ident) = if input.peek(LitStr) {
                (input.parse::<LitStr>()?.value(), None)
            } else if input.peek(Ident) {
                let ident = input.parse::<Ident>()?;
                (ident.to_string(), Some(ident))
            } else {
                return Err(input.error("expected object property key"));
            };

            let value = if input.peek(Token![:]) {
                input.parse::<Token![:]>()?;
                input.parse()?
            } else if let Some(ident) = shorthand_ident {
                JsExpression::Identifier(ident)
            } else {
                return Err(input.error("quoted object property keys require a value"));
            };

            Ok(JsProperty { key, value })
        }
    }
}

mod ir;

#[proc_macro]
pub fn js(input: TokenStream) -> TokenStream {
    let script = parse_macro_input!(input as parser::JsScript);

    let runtime_statements: Vec<_> = script
        .statements
        .iter()
        .map(ir::convert_statement_to_runtime)
        .collect();

    let expanded = quote! {
        {
            use js_runtime::{JsExecutionContext, JsProgram, JsScope};

            let scope = JsScope::new_root();
            let context = JsExecutionContext {
                scope,
                this_context: None
            };

            let program = JsProgram::new(vec![#(#runtime_statements),*]);
            program.execute(&context);
        }
    };

    TokenStream::from(expanded)
}
