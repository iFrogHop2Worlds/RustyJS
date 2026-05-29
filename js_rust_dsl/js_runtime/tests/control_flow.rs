use js_runtime::{
    JsDeclarationKind, JsExecutionContext, JsExpression, JsProgram, JsScope, JsStatement, JsValue,
};

fn root_context() -> JsExecutionContext {
    JsExecutionContext {
        scope: JsScope::new_root(),
        this_context: None,
    }
}

#[test]
fn return_short_circuits_function_body() {
    let context = root_context();
    let program = JsProgram::new(vec![JsStatement::TryCatchFinally {
        try_block: vec![JsStatement::Return(Some(JsExpression::Literal(
            JsValue::Number(1.0),
        )))],
        catch_param: None,
        catch_block: None,
        finally_block: Some(vec![JsStatement::Expression(JsExpression::Literal(
            JsValue::Number(2.0),
        ))]),
    }]);

    let result = program.execute(&context);
    assert!(matches!(result, JsValue::Number(n) if (n - 1.0).abs() < f64::EPSILON));
}

#[test]
fn if_else_selects_expected_branch() {
    let context = root_context();
    let program = JsProgram::new(vec![JsStatement::IfStatement(
        JsExpression::Literal(JsValue::Boolean(false)),
        vec![JsStatement::Expression(JsExpression::Literal(
            JsValue::Number(1.0),
        ))],
        Some(vec![JsStatement::Expression(JsExpression::Literal(
            JsValue::Number(2.0),
        ))]),
    )]);

    let result = program.execute(&context);
    assert!(matches!(result, JsValue::Number(n) if (n - 2.0).abs() < f64::EPSILON));
}

#[test]
fn for_loop_updates_existing_binding() {
    let context = root_context();
    let program = JsProgram::new(vec![
        JsStatement::Declaration {
            kind: JsDeclarationKind::Let,
            name: "x".to_string(),
            expr: JsExpression::Literal(JsValue::Number(0.0)),
        },
        JsStatement::ForLoop {
            init: Some(Box::new(JsStatement::Declaration {
                kind: JsDeclarationKind::Let,
                name: "i".to_string(),
                expr: JsExpression::Literal(JsValue::Number(0.0)),
            })),
            condition: Some(JsExpression::BinaryOp(
                Box::new(JsExpression::Identifier("i".to_string())),
                "<".to_string(),
                Box::new(JsExpression::Literal(JsValue::Number(3.0))),
            )),
            increment: Some(Box::new(JsStatement::Assignment(
                "i".to_string(),
                JsExpression::BinaryOp(
                    Box::new(JsExpression::Identifier("i".to_string())),
                    "+".to_string(),
                    Box::new(JsExpression::Literal(JsValue::Number(1.0))),
                ),
            ))),
            body: vec![JsStatement::Assignment(
                "x".to_string(),
                JsExpression::BinaryOp(
                    Box::new(JsExpression::Identifier("x".to_string())),
                    "+".to_string(),
                    Box::new(JsExpression::Literal(JsValue::Number(2.0))),
                ),
            )],
        },
        JsStatement::Expression(JsExpression::Identifier("x".to_string())),
    ]);

    let result = program.execute(&context);
    assert!(matches!(result, JsValue::Number(n) if (n - 6.0).abs() < f64::EPSILON));
}

#[test]
fn member_and_index_assignments_update_values() {
    let context = root_context();
    let program = JsProgram::new(vec![
        JsStatement::Declaration {
            kind: JsDeclarationKind::Let,
            name: "obj".to_string(),
            expr: JsExpression::ObjectLiteral(vec![(
                "count".to_string(),
                JsExpression::Literal(JsValue::Number(1.0)),
            )]),
        },
        JsStatement::MemberAssignment(
            JsExpression::Identifier("obj".to_string()),
            "count".to_string(),
            JsExpression::BinaryOp(
                Box::new(JsExpression::MemberAccess(
                    Box::new(JsExpression::Identifier("obj".to_string())),
                    "count".to_string(),
                )),
                "+".to_string(),
                Box::new(JsExpression::Literal(JsValue::Number(2.0))),
            ),
        ),
        JsStatement::Declaration {
            kind: JsDeclarationKind::Let,
            name: "arr".to_string(),
            expr: JsExpression::ArrayLiteral(vec![JsExpression::Literal(JsValue::Number(3.0))]),
        },
        JsStatement::IndexAssignment(
            JsExpression::Identifier("arr".to_string()),
            JsExpression::Literal(JsValue::Number(0.0)),
            JsExpression::BinaryOp(
                Box::new(JsExpression::IndexAccess(
                    Box::new(JsExpression::Identifier("arr".to_string())),
                    Box::new(JsExpression::Literal(JsValue::Number(0.0))),
                )),
                "+".to_string(),
                Box::new(JsExpression::Literal(JsValue::Number(4.0))),
            ),
        ),
        JsStatement::Expression(JsExpression::BinaryOp(
            Box::new(JsExpression::MemberAccess(
                Box::new(JsExpression::Identifier("obj".to_string())),
                "count".to_string(),
            )),
            "+".to_string(),
            Box::new(JsExpression::IndexAccess(
                Box::new(JsExpression::Identifier("arr".to_string())),
                Box::new(JsExpression::Literal(JsValue::Number(0.0))),
            )),
        )),
    ]);

    let result = program.execute(&context);
    assert!(matches!(result, JsValue::Number(n) if (n - 10.0).abs() < f64::EPSILON));
}
