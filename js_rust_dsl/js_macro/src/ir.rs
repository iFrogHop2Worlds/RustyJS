use quote::quote;

use crate::parser;

#[derive(Clone, Debug)]
pub enum IrDeclarationKind {
    Let,
    Const,
}

#[derive(Clone, Debug)]
pub enum IrStatement {
    Expression(IrExpression),
    Assignment(String, IrExpression),
    Declaration {
        kind: IrDeclarationKind,
        name: String,
        expr: IrExpression,
    },
    MemberAssignment(IrExpression, String, IrExpression),
    IndexAssignment(IrExpression, IrExpression, IrExpression),
    Return(Option<IrExpression>),
    IfStatement(IrExpression, Vec<IrStatement>, Option<Vec<IrStatement>>),
    WhileLoop(IrExpression, Vec<IrStatement>),
    DoWhileLoop(Vec<IrStatement>, IrExpression),
    ForLoop {
        init: Option<Box<IrStatement>>,
        condition: Option<IrExpression>,
        increment: Option<Box<IrStatement>>,
        body: Vec<IrStatement>,
    },
    TryCatchFinally {
        try_block: Vec<IrStatement>,
        catch_param: Option<String>,
        catch_block: Option<Vec<IrStatement>>,
        finally_block: Option<Vec<IrStatement>>,
    },
    Throw(IrExpression),
}

#[derive(Clone, Debug)]
pub enum IrExpression {
    Literal(IrValue),
    Identifier(String),
    This,
    BinaryOp(Box<IrExpression>, String, Box<IrExpression>),
    UnaryOp(String, Box<IrExpression>),
    LogicalOp(Box<IrExpression>, String, Box<IrExpression>),
    Call(String, Vec<IrExpression>),
    MemberAccess(Box<IrExpression>, String),
    IndexAccess(Box<IrExpression>, Box<IrExpression>),
    MethodCall {
        object: Box<IrExpression>,
        method: String,
        args: Vec<IrExpression>,
    },
}

#[derive(Clone, Debug)]
pub struct IrFunction {
    pub parameters: Vec<String>,
    pub body: Vec<IrStatement>,
    pub name: String,
}

#[derive(Clone, Debug)]
pub enum IrValue {
    Number(f64),
    String(String),
    Boolean(bool),
    Null,
    Undefined,
    Object(Vec<(String, IrExpression)>),
    Array(Vec<IrExpression>),
    Function(IrFunction),
}

fn lower_expression_to_ir(expr: &parser::JsExpression) -> IrExpression {
    match expr {
        parser::JsExpression::Literal(lit) => {
            let ir_val = match lit {
                parser::JsLiteral::String(s) => IrValue::String(s.clone()),
                parser::JsLiteral::Number(n) => IrValue::Number(*n),
                parser::JsLiteral::Boolean(b) => IrValue::Boolean(*b),
                parser::JsLiteral::Null => IrValue::Null,
                parser::JsLiteral::Undefined => IrValue::Undefined,
                parser::JsLiteral::Function(func) => IrValue::Function(IrFunction {
                    parameters: func.params.iter().map(|p| p.to_string()).collect(),
                    body: func
                        .body
                        .statements
                        .iter()
                        .map(lower_statement_to_ir)
                        .collect(),
                    name: func
                        .name
                        .as_ref()
                        .map(|n| n.to_string())
                        .unwrap_or_else(|| "anonymous".to_string()),
                }),
            };
            IrExpression::Literal(ir_val)
        }
        parser::JsExpression::Identifier(ident) => IrExpression::Identifier(ident.to_string()),
        parser::JsExpression::This => IrExpression::This,
        parser::JsExpression::BinaryOp(left, op, right) => {
            let op_str = match op {
                parser::JsBinaryOp::Add => "+",
                parser::JsBinaryOp::Subtract => "-",
                parser::JsBinaryOp::Multiply => "*",
                parser::JsBinaryOp::Equal => "==",
                parser::JsBinaryOp::NotEqual => "!=",
                parser::JsBinaryOp::StrictEqual => "===",
                parser::JsBinaryOp::StrictNotEqual => "!==",
                parser::JsBinaryOp::LessThan => "<",
                parser::JsBinaryOp::GreaterThan => ">",
                parser::JsBinaryOp::LessThanOrEqual => "<=",
                parser::JsBinaryOp::GreaterThanOrEqual => ">=",
            };
            IrExpression::BinaryOp(
                Box::new(lower_expression_to_ir(left)),
                op_str.to_string(),
                Box::new(lower_expression_to_ir(right)),
            )
        }
        parser::JsExpression::UnaryOp(op, expr) => {
            let op_str = match op {
                parser::JsUnaryOp::Negate => "-",
                parser::JsUnaryOp::Not => "!",
            };
            IrExpression::UnaryOp(op_str.to_string(), Box::new(lower_expression_to_ir(expr)))
        }
        parser::JsExpression::LogicalOp(left, op, right) => {
            let op_str = match op {
                parser::JsLogicalOp::And => "&&",
                parser::JsLogicalOp::Or => "||",
            };
            IrExpression::LogicalOp(
                Box::new(lower_expression_to_ir(left)),
                op_str.to_string(),
                Box::new(lower_expression_to_ir(right)),
            )
        }
        parser::JsExpression::Assignment(_, _)
        | parser::JsExpression::MemberAssignment(_, _, _)
        | parser::JsExpression::IndexAssignment(_, _, _) => {
            IrExpression::Literal(IrValue::Undefined)
        }
        parser::JsExpression::Call(ident, args) => IrExpression::Call(
            ident.to_string(),
            args.iter().map(lower_expression_to_ir).collect(),
        ),
        parser::JsExpression::Array(elements) => IrExpression::Literal(IrValue::Array(
            elements.iter().map(lower_expression_to_ir).collect(),
        )),
        parser::JsExpression::Object(obj) => {
            let properties = obj
                .properties
                .iter()
                .map(|p| (p.key.to_string(), lower_expression_to_ir(&p.value)))
                .collect();
            IrExpression::Literal(IrValue::Object(properties))
        }
        parser::JsExpression::MemberAccess(obj, member) => {
            IrExpression::MemberAccess(Box::new(lower_expression_to_ir(obj)), member.to_string())
        }
        parser::JsExpression::IndexAccess(obj, index) => IrExpression::IndexAccess(
            Box::new(lower_expression_to_ir(obj)),
            Box::new(lower_expression_to_ir(index)),
        ),
        parser::JsExpression::MethodCall {
            object,
            method,
            args,
        } => IrExpression::MethodCall {
            object: Box::new(lower_expression_to_ir(object)),
            method: method.to_string(),
            args: args.iter().map(lower_expression_to_ir).collect(),
        },
    }
}

fn lower_statement_to_ir(stmt: &parser::JsStatement) -> IrStatement {
    match stmt {
        parser::JsStatement::Expression(expr) => match expr {
            parser::JsExpression::Assignment(ident, value_expr) => {
                IrStatement::Assignment(ident.to_string(), lower_expression_to_ir(value_expr))
            }
            parser::JsExpression::MemberAssignment(obj, prop, value) => {
                IrStatement::MemberAssignment(
                    lower_expression_to_ir(obj),
                    prop.to_string(),
                    lower_expression_to_ir(value),
                )
            }
            parser::JsExpression::IndexAssignment(obj, index, value) => {
                IrStatement::IndexAssignment(
                    lower_expression_to_ir(obj),
                    lower_expression_to_ir(index),
                    lower_expression_to_ir(value),
                )
            }
            _ => IrStatement::Expression(lower_expression_to_ir(expr)),
        },
        parser::JsStatement::LetDecl(ident, expr) => IrStatement::Declaration {
            kind: IrDeclarationKind::Let,
            name: ident.to_string(),
            expr: lower_expression_to_ir(expr),
        },
        parser::JsStatement::ConstDecl(ident, expr) => IrStatement::Declaration {
            kind: IrDeclarationKind::Const,
            name: ident.to_string(),
            expr: lower_expression_to_ir(expr),
        },
        parser::JsStatement::FunctionDecl(func) => {
            if let Some(name) = &func.name {
                IrStatement::Declaration {
                    kind: IrDeclarationKind::Let,
                    name: name.to_string(),
                    expr: IrExpression::Literal(IrValue::Function(IrFunction {
                        parameters: func.params.iter().map(|p| p.to_string()).collect(),
                        body: func
                            .body
                            .statements
                            .iter()
                            .map(lower_statement_to_ir)
                            .collect(),
                        name: name.to_string(),
                    })),
                }
            } else {
                IrStatement::Expression(IrExpression::Literal(IrValue::Undefined))
            }
        }
        parser::JsStatement::Return(expr_opt) => {
            IrStatement::Return(expr_opt.as_ref().map(lower_expression_to_ir))
        }
        parser::JsStatement::IfStatement(cond, body, else_body) => IrStatement::IfStatement(
            lower_expression_to_ir(cond),
            body.statements.iter().map(lower_statement_to_ir).collect(),
            else_body
                .as_ref()
                .map(|b| b.statements.iter().map(lower_statement_to_ir).collect()),
        ),
        parser::JsStatement::WhileLoop(while_loop) => IrStatement::WhileLoop(
            lower_expression_to_ir(&while_loop.condition),
            while_loop
                .body
                .statements
                .iter()
                .map(lower_statement_to_ir)
                .collect(),
        ),
        parser::JsStatement::DoWhileLoop(do_while) => IrStatement::DoWhileLoop(
            do_while
                .body
                .statements
                .iter()
                .map(lower_statement_to_ir)
                .collect(),
            lower_expression_to_ir(&do_while.condition),
        ),
        parser::JsStatement::ForLoop(for_loop) => IrStatement::ForLoop {
            init: for_loop
                .init
                .as_ref()
                .map(|init| Box::new(lower_statement_to_ir(init))),
            condition: for_loop.condition.as_ref().map(lower_expression_to_ir),
            increment: for_loop.increment.as_ref().map(|incr| match incr.as_ref() {
                parser::JsExpression::Assignment(ident, value_expr) => Box::new(
                    IrStatement::Assignment(ident.to_string(), lower_expression_to_ir(value_expr)),
                ),
                _ => Box::new(IrStatement::Expression(lower_expression_to_ir(incr))),
            }),
            body: for_loop
                .body
                .statements
                .iter()
                .map(lower_statement_to_ir)
                .collect(),
        },
        parser::JsStatement::TryCatchFinally {
            try_block,
            catch_param,
            catch_block,
            finally_block,
        } => IrStatement::TryCatchFinally {
            try_block: try_block
                .statements
                .iter()
                .map(lower_statement_to_ir)
                .collect(),
            catch_param: catch_param.as_ref().map(|p| p.to_string()),
            catch_block: catch_block
                .as_ref()
                .map(|b| b.statements.iter().map(lower_statement_to_ir).collect()),
            finally_block: finally_block
                .as_ref()
                .map(|b| b.statements.iter().map(lower_statement_to_ir).collect()),
        },
        parser::JsStatement::Throw(expr) => IrStatement::Throw(lower_expression_to_ir(expr)),
    }
}

fn emit_ir_statement_to_runtime(stmt: &IrStatement) -> proc_macro2::TokenStream {
    match stmt {
        IrStatement::Expression(expr) => {
            let runtime_expr = emit_ir_expression_to_runtime(expr);
            quote! { js_runtime::JsStatement::Expression(#runtime_expr) }
        }
        IrStatement::Assignment(name, expr) => {
            let runtime_expr = emit_ir_expression_to_runtime(expr);
            quote! { js_runtime::JsStatement::Assignment(#name.to_string(), #runtime_expr) }
        }
        IrStatement::Declaration { kind, name, expr } => {
            let runtime_expr = emit_ir_expression_to_runtime(expr);
            let runtime_kind = match kind {
                IrDeclarationKind::Let => quote! { js_runtime::JsDeclarationKind::Let },
                IrDeclarationKind::Const => quote! { js_runtime::JsDeclarationKind::Const },
            };
            quote! {
                js_runtime::JsStatement::Declaration {
                    kind: #runtime_kind,
                    name: #name.to_string(),
                    expr: #runtime_expr
                }
            }
        }
        IrStatement::MemberAssignment(obj, prop, value) => {
            let obj_runtime = emit_ir_expression_to_runtime(obj);
            let value_runtime = emit_ir_expression_to_runtime(value);
            quote! {
                js_runtime::JsStatement::MemberAssignment(
                    #obj_runtime,
                    #prop.to_string(),
                    #value_runtime
                )
            }
        }
        IrStatement::IndexAssignment(obj, index, value) => {
            let obj_runtime = emit_ir_expression_to_runtime(obj);
            let index_runtime = emit_ir_expression_to_runtime(index);
            let value_runtime = emit_ir_expression_to_runtime(value);
            quote! {
                js_runtime::JsStatement::IndexAssignment(
                    #obj_runtime,
                    #index_runtime,
                    #value_runtime
                )
            }
        }
        IrStatement::Return(expr_opt) => {
            if let Some(expr) = expr_opt {
                let runtime_expr = emit_ir_expression_to_runtime(expr);
                quote! { js_runtime::JsStatement::Return(Some(#runtime_expr)) }
            } else {
                quote! { js_runtime::JsStatement::Return(None) }
            }
        }
        IrStatement::IfStatement(cond, body, else_body) => {
            let runtime_cond = emit_ir_expression_to_runtime(cond);
            let runtime_body: Vec<_> = body.iter().map(emit_ir_statement_to_runtime).collect();
            let runtime_else = if let Some(else_stmts) = else_body {
                let emitted: Vec<_> = else_stmts
                    .iter()
                    .map(emit_ir_statement_to_runtime)
                    .collect();
                quote! { Some(vec![#(#emitted),*]) }
            } else {
                quote! { None }
            };
            quote! { js_runtime::JsStatement::IfStatement(#runtime_cond, vec![#(#runtime_body),*], #runtime_else) }
        }
        IrStatement::WhileLoop(cond, body) => {
            let runtime_cond = emit_ir_expression_to_runtime(cond);
            let runtime_body: Vec<_> = body.iter().map(emit_ir_statement_to_runtime).collect();
            quote! { js_runtime::JsStatement::WhileLoop(#runtime_cond, vec![#(#runtime_body),*]) }
        }
        IrStatement::DoWhileLoop(body, cond) => {
            let runtime_cond = emit_ir_expression_to_runtime(cond);
            let runtime_body: Vec<_> = body.iter().map(emit_ir_statement_to_runtime).collect();
            quote! { js_runtime::JsStatement::DoWhileLoop(vec![#(#runtime_body),*], #runtime_cond) }
        }
        IrStatement::ForLoop {
            init,
            condition,
            increment,
            body,
        } => {
            let runtime_init = init
                .as_ref()
                .map(|s| {
                    let t = emit_ir_statement_to_runtime(s);
                    quote! { Some(Box::new(#t)) }
                })
                .unwrap_or_else(|| quote! { None });
            let runtime_condition = condition
                .as_ref()
                .map(|c| {
                    let t = emit_ir_expression_to_runtime(c);
                    quote! { Some(#t) }
                })
                .unwrap_or_else(|| quote! { None });
            let runtime_increment = increment
                .as_ref()
                .map(|s| {
                    let t = emit_ir_statement_to_runtime(s);
                    quote! { Some(Box::new(#t)) }
                })
                .unwrap_or_else(|| quote! { None });
            let runtime_body: Vec<_> = body.iter().map(emit_ir_statement_to_runtime).collect();
            quote! {
                js_runtime::JsStatement::ForLoop {
                    init: #runtime_init,
                    condition: #runtime_condition,
                    increment: #runtime_increment,
                    body: vec![#(#runtime_body),*],
                }
            }
        }
        IrStatement::TryCatchFinally {
            try_block,
            catch_param,
            catch_block,
            finally_block,
        } => {
            let runtime_try: Vec<_> = try_block.iter().map(emit_ir_statement_to_runtime).collect();
            let runtime_catch_param = catch_param
                .as_ref()
                .map(|p| quote! { Some(#p.to_string()) })
                .unwrap_or_else(|| quote! { None });
            let runtime_catch_block = catch_block
                .as_ref()
                .map(|b| {
                    let emitted: Vec<_> = b.iter().map(emit_ir_statement_to_runtime).collect();
                    quote! { Some(vec![#(#emitted),*]) }
                })
                .unwrap_or_else(|| quote! { None });
            let runtime_finally_block = finally_block
                .as_ref()
                .map(|b| {
                    let emitted: Vec<_> = b.iter().map(emit_ir_statement_to_runtime).collect();
                    quote! { Some(vec![#(#emitted),*]) }
                })
                .unwrap_or_else(|| quote! { None });
            quote! {
                js_runtime::JsStatement::TryCatchFinally {
                    try_block: vec![#(#runtime_try),*],
                    catch_param: #runtime_catch_param,
                    catch_block: #runtime_catch_block,
                    finally_block: #runtime_finally_block,
                }
            }
        }
        IrStatement::Throw(expr) => {
            let runtime_expr = emit_ir_expression_to_runtime(expr);
            quote! { js_runtime::JsStatement::Throw(#runtime_expr) }
        }
    }
}

fn emit_ir_expression_to_runtime(expr: &IrExpression) -> proc_macro2::TokenStream {
    match expr {
        IrExpression::Literal(lit) => match lit {
            IrValue::String(s) => {
                quote! { js_runtime::JsExpression::Literal(js_runtime::JsValue::String(#s.to_string())) }
            }
            IrValue::Number(n) => {
                quote! { js_runtime::JsExpression::Literal(js_runtime::JsValue::Number(#n)) }
            }
            IrValue::Boolean(b) => {
                quote! { js_runtime::JsExpression::Literal(js_runtime::JsValue::Boolean(#b)) }
            }
            IrValue::Null => {
                quote! { js_runtime::JsExpression::Literal(js_runtime::JsValue::Null) }
            }
            IrValue::Undefined => {
                quote! { js_runtime::JsExpression::Literal(js_runtime::JsValue::Undefined) }
            }
            IrValue::Function(func) => {
                let params = &func.parameters;
                let name = &func.name;
                let runtime_body: Vec<_> =
                    func.body.iter().map(emit_ir_statement_to_runtime).collect();
                quote! {
                    js_runtime::JsExpression::Literal(
                        js_runtime::JsValue::Function(
                            js_runtime::JsFunction::new(
                                vec![#(#params.to_string()),*],
                                vec![#(#runtime_body),*],
                                #name.to_string()
                            )
                        )
                    )
                }
            }
            IrValue::Object(props) => {
                let entries = props.iter().map(|(k, v)| {
                    let rv = emit_ir_expression_to_runtime(v);
                    quote! { (#k.to_string(), #rv.evaluate(&temp_context)) }
                });
                quote! {
                    js_runtime::JsExpression::Literal({
                        let temp_context = js_runtime::JsExecutionContext {
                            scope: js_runtime::JsScope::new_root(),
                            this_context: None
                        };
                        let mut map = std::collections::HashMap::new();
                        for (key, value) in vec![#(#entries),*] {
                            map.insert(key, value);
                        }
                        js_runtime::JsValue::Object(map)
                    })
                }
            }
            IrValue::Array(elements) => {
                let emitted: Vec<_> = elements.iter().map(emit_ir_expression_to_runtime).collect();
                quote! {
                    js_runtime::JsExpression::Literal(
                        js_runtime::JsValue::Array({
                            let temp_context = js_runtime::JsExecutionContext {
                                scope: js_runtime::JsScope::new_root(),
                                this_context: None
                            };
                            vec![#(#emitted.evaluate(&temp_context)),*]
                        })
                    )
                }
            }
        },
        IrExpression::Identifier(name) => {
            quote! { js_runtime::JsExpression::Identifier(#name.to_string()) }
        }
        IrExpression::This => quote! { js_runtime::JsExpression::This },
        IrExpression::BinaryOp(left, op, right) => {
            let l = emit_ir_expression_to_runtime(left);
            let r = emit_ir_expression_to_runtime(right);
            quote! { js_runtime::JsExpression::BinaryOp(Box::new(#l), #op.to_string(), Box::new(#r)) }
        }
        IrExpression::UnaryOp(op, expr) => {
            let e = emit_ir_expression_to_runtime(expr);
            quote! { js_runtime::JsExpression::UnaryOp(#op.to_string(), Box::new(#e)) }
        }
        IrExpression::LogicalOp(left, op, right) => {
            let l = emit_ir_expression_to_runtime(left);
            let r = emit_ir_expression_to_runtime(right);
            quote! { js_runtime::JsExpression::LogicalOp(Box::new(#l), #op.to_string(), Box::new(#r)) }
        }
        IrExpression::Call(name, args) => {
            let emitted: Vec<_> = args.iter().map(emit_ir_expression_to_runtime).collect();
            quote! { js_runtime::JsExpression::Call(#name.to_string(), vec![#(#emitted),*]) }
        }
        IrExpression::MemberAccess(obj, member) => {
            let o = emit_ir_expression_to_runtime(obj);
            quote! { js_runtime::JsExpression::MemberAccess(Box::new(#o), #member.to_string()) }
        }
        IrExpression::IndexAccess(obj, index) => {
            let o = emit_ir_expression_to_runtime(obj);
            let i = emit_ir_expression_to_runtime(index);
            quote! { js_runtime::JsExpression::IndexAccess(Box::new(#o), Box::new(#i)) }
        }
        IrExpression::MethodCall {
            object,
            method,
            args,
        } => {
            let o = emit_ir_expression_to_runtime(object);
            let emitted: Vec<_> = args.iter().map(emit_ir_expression_to_runtime).collect();
            quote! {
                js_runtime::JsExpression::MethodCall {
                    object: Box::new(#o),
                    method: #method.to_string(),
                    args: vec![#(#emitted),*]
                }
            }
        }
    }
}

pub(crate) fn convert_statement_to_runtime(stmt: &parser::JsStatement) -> proc_macro2::TokenStream {
    let ir_stmt = lower_statement_to_ir(stmt);
    emit_ir_statement_to_runtime(&ir_stmt)
}
