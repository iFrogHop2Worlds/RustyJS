use js_runtime::{JsExecutionContext, JsExpression, JsProgram, JsScope, JsStatement, JsValue};

fn root_context() -> JsExecutionContext {
    JsExecutionContext {
        scope: JsScope::new_root(),
        this_context: None,
    }
}

#[test]
fn error_constructor_returns_error_object() {
    let context = root_context();
    let program = JsProgram::new(vec![JsStatement::Expression(JsExpression::Call(
        "Error".to_string(),
        vec![JsExpression::Literal(JsValue::String("boom".to_string()))],
    ))]);

    let result = program.execute(&context);
    assert!(matches!(
        result.get_property("name"),
        JsValue::String(name) if name == "Error"
    ));
    assert!(matches!(
        result.get_property("message"),
        JsValue::String(message) if message == "boom"
    ));
}

#[test]
fn catch_without_binding_handles_exception() {
    let context = root_context();
    let program = JsProgram::new(vec![JsStatement::TryCatchFinally {
        try_block: vec![JsStatement::Throw(JsExpression::Call(
            "TypeError".to_string(),
            vec![JsExpression::Literal(JsValue::String("bad".to_string()))],
        ))],
        catch_param: None,
        catch_block: Some(vec![JsStatement::Expression(JsExpression::Literal(
            JsValue::String("handled".to_string()),
        ))]),
        finally_block: None,
    }]);

    let result = program.execute(&context);
    assert!(matches!(result, JsValue::String(value) if value == "handled"));
}

#[test]
fn catch_binding_can_read_error_message() {
    let context = root_context();
    let program = JsProgram::new(vec![JsStatement::TryCatchFinally {
        try_block: vec![JsStatement::Throw(JsExpression::Call(
            "Error".to_string(),
            vec![JsExpression::Literal(JsValue::String("kaboom".to_string()))],
        ))],
        catch_param: Some("e".to_string()),
        catch_block: Some(vec![JsStatement::Expression(JsExpression::MemberAccess(
            Box::new(JsExpression::Identifier("e".to_string())),
            "message".to_string(),
        ))]),
        finally_block: None,
    }]);

    let result = program.execute(&context);
    assert!(matches!(result, JsValue::String(message) if message == "kaboom"));
}

#[test]
fn finally_preserves_rethrow_when_it_completes_normally() {
    let context = root_context();
    let program = JsProgram::new(vec![JsStatement::TryCatchFinally {
        try_block: vec![JsStatement::Throw(JsExpression::Literal(JsValue::String(
            "original".to_string(),
        )))],
        catch_param: Some("e".to_string()),
        catch_block: Some(vec![JsStatement::Throw(JsExpression::Identifier(
            "e".to_string(),
        ))]),
        finally_block: Some(vec![JsStatement::Expression(JsExpression::Literal(
            JsValue::String("cleanup".to_string()),
        ))]),
    }]);

    let result = program.execute(&context);
    assert!(matches!(result, JsValue::String(value) if value == "original"));
}

#[test]
fn finally_throw_overrides_prior_return() {
    let context = root_context();
    let program = JsProgram::new(vec![JsStatement::TryCatchFinally {
        try_block: vec![JsStatement::Return(Some(JsExpression::Literal(
            JsValue::Number(1.0),
        )))],
        catch_param: None,
        catch_block: None,
        finally_block: Some(vec![JsStatement::Throw(JsExpression::Literal(
            JsValue::String("finally".to_string()),
        ))]),
    }]);

    let result = program.execute(&context);
    assert!(matches!(result, JsValue::String(value) if value == "finally"));
}
