use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

pub mod builtins;

#[derive(Debug, Default)]
pub struct JsScope {
    bindings: HashMap<String, JsBinding>,
    parent: Option<Rc<RefCell<JsScope>>>,
}

#[derive(Clone, Debug)]
struct JsBinding {
    value: JsValue,
    mutable: bool,
}

#[derive(Clone, Copy, Debug)]
pub enum JsDeclarationKind {
    Let,
    Const,
}

impl JsScope {
    pub fn new_root() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            bindings: HashMap::new(),
            parent: None,
        }))
    }

    pub fn new_child(parent: Rc<RefCell<JsScope>>) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            bindings: HashMap::new(),
            parent: Some(parent),
        }))
    }

    fn define(&mut self, name: String, value: JsValue, kind: JsDeclarationKind) {
        self.bindings.insert(
            name,
            JsBinding {
                value,
                mutable: matches!(kind, JsDeclarationKind::Let),
            },
        );
    }

    fn get(&self, name: &str) -> Option<JsValue> {
        if let Some(binding) = self.bindings.get(name) {
            return Some(binding.value.clone());
        }

        self.parent
            .as_ref()
            .and_then(|parent| parent.borrow().get(name))
    }

    fn set(&mut self, name: &str, value: JsValue) -> bool {
        if let Some(binding) = self.bindings.get_mut(name) {
            if binding.mutable {
                binding.value = value;
                return true;
            }
            return false;
        }

        if let Some(parent) = &self.parent {
            if parent.borrow().get(name).is_some() {
                return parent.borrow_mut().set(name, value);
            }
        }

        self.bindings.insert(
            name.to_string(),
            JsBinding {
                value,
                mutable: true,
            },
        );
        true
    }
}

pub struct JsExecutionContext {
    pub scope: Rc<RefCell<JsScope>>,
    pub this_context: Option<Rc<RefCell<JsValue>>>,
}

enum StatementFlow {
    Continue(JsValue),
    Return(JsValue),
    Throw(JsValue),
}

enum VmFlow {
    Continue(JsValue),
    Return(JsValue),
    Throw(JsValue),
}

#[derive(Clone, Debug)]
enum Instruction {
    PushLiteral(JsValue),
    PushUndefined,
    LoadIdentifier(String),
    LoadThis,
    StoreIdentifier(String),
    Declare {
        kind: JsDeclarationKind,
        name: String,
    },
    Dup,
    Pop,
    SetLast,
    BinaryOp(String),
    UnaryOp(String),
    LogicalAnd(usize),
    LogicalOr(usize),
    Call(String, usize),
    MemberAccess(String),
    IndexAccess,
    MethodCall {
        method: String,
        arg_count: usize,
    },
    MethodCallNamed {
        object_name: String,
        method: String,
        arg_count: usize,
    },
    AssignThisMember(String),
    AssignNamedMember {
        object_name: String,
        member: String,
    },
    AssignComputedMember(String),
    AssignNamedIndex(String),
    AssignComputedIndex,
    JumpIfFalse(usize),
    Jump(usize),
    Throw,
    TryCatchFinally {
        try_program: BytecodeProgram,
        catch_param: Option<String>,
        catch_program: Option<BytecodeProgram>,
        finally_program: Option<BytecodeProgram>,
    },
    Return,
}

#[derive(Clone, Debug)]
struct BytecodeProgram {
    instructions: Vec<Instruction>,
}

#[derive(Clone, Debug)]
pub enum JsStatement {
    Expression(JsExpression),
    Assignment(String, JsExpression),
    Declaration {
        kind: JsDeclarationKind,
        name: String,
        expr: JsExpression,
    },
    MemberAssignment(JsExpression, String, JsExpression),
    IndexAssignment(JsExpression, JsExpression, JsExpression),
    Return(Option<JsExpression>),
    IfStatement(JsExpression, Vec<JsStatement>, Option<Vec<JsStatement>>),
    WhileLoop(JsExpression, Vec<JsStatement>),
    DoWhileLoop(Vec<JsStatement>, JsExpression),
    ForLoop {
        init: Option<Box<JsStatement>>,
        condition: Option<JsExpression>,
        increment: Option<Box<JsStatement>>,
        body: Vec<JsStatement>,
    },
    TryCatchFinally {
        try_block: Vec<JsStatement>,
        catch_param: Option<String>,
        catch_block: Option<Vec<JsStatement>>,
        finally_block: Option<Vec<JsStatement>>,
    },
    Throw(JsExpression),
}

#[derive(Clone, Debug)]
pub struct JsProgram {
    bytecode: BytecodeProgram,
}

impl JsProgram {
    pub fn new(statements: Vec<JsStatement>) -> Self {
        Self {
            bytecode: compile_program(&statements),
        }
    }

    pub fn execute(&self, context: &JsExecutionContext) -> JsValue {
        execute_bytecode(&self.bytecode, context)
    }
}

#[derive(Clone, Debug)]
pub enum JsExpression {
    Literal(JsValue),
    Identifier(String),
    This,
    BinaryOp(Box<JsExpression>, String, Box<JsExpression>),
    UnaryOp(String, Box<JsExpression>),
    LogicalOp(Box<JsExpression>, String, Box<JsExpression>),
    Call(String, Vec<JsExpression>),
    MemberAccess(Box<JsExpression>, String),
    IndexAccess(Box<JsExpression>, Box<JsExpression>),
    MethodCall {
        object: Box<JsExpression>,
        method: String,
        args: Vec<JsExpression>,
    },
}

#[derive(Clone, Debug)]
pub struct JsFunction {
    pub parameters: Vec<String>,
    bytecode: BytecodeProgram,
}

#[derive(Clone, Debug)]
pub enum JsErrorKind {
    Error,
    TypeError,
}

impl JsErrorKind {
    fn as_str(&self) -> &'static str {
        match self {
            JsErrorKind::Error => "Error",
            JsErrorKind::TypeError => "TypeError",
        }
    }
}

#[derive(Clone, Debug)]
pub struct JsError {
    pub kind: JsErrorKind,
    pub message: String,
    pub stack: Option<String>,
    pub properties: HashMap<String, JsValue>,
}

#[derive(Clone, Debug)]
pub enum JsValue {
    Number(f64),
    String(String),
    Boolean(bool),
    Null,
    Undefined,
    Object(HashMap<String, JsValue>),
    Array(Vec<JsValue>),
    Function(JsFunction),
    Error(JsError),
}

impl JsFunction {
    pub fn new(parameters: Vec<String>, body: Vec<JsStatement>, _name: String) -> Self {
        Self {
            parameters,
            bytecode: compile_program(&body),
        }
    }

    pub fn execute(
        &self,
        args: &[JsValue],
        global_scope: Rc<RefCell<JsScope>>,
        this_context: Option<Rc<RefCell<JsValue>>>,
    ) -> JsValue {
        let func_scope = JsScope::new_child(global_scope);
        for (i, param_name) in self.parameters.iter().enumerate() {
            if let Some(arg) = args.get(i) {
                func_scope.borrow_mut().define(
                    param_name.clone(),
                    arg.clone(),
                    JsDeclarationKind::Let,
                );
            }
        }

        let context = JsExecutionContext {
            scope: func_scope,
            this_context,
        };

        execute_bytecode(&self.bytecode, &context)
    }
}

impl JsExpression {
    pub fn evaluate(&self, context: &JsExecutionContext) -> JsValue {
        match self {
            JsExpression::Literal(value) => value.clone(),
            JsExpression::Identifier(name) => {
                if let Some(value) = builtins::resolve_global(name) {
                    return value;
                }

                context
                    .scope
                    .borrow()
                    .get(name)
                    .unwrap_or(JsValue::Undefined)
            }
            JsExpression::This => context
                .this_context
                .as_ref()
                .map(|this_ref| this_ref.borrow().clone())
                .unwrap_or(JsValue::Undefined),
            JsExpression::BinaryOp(left, op, right) => {
                let left_val = left.evaluate(context);
                let right_val = right.evaluate(context);
                match op.as_str() {
                    "+" => left_val.add(&right_val),
                    "-" => JsValue::Number(left_val.to_number() - right_val.to_number()),
                    "*" => match (&left_val, &right_val) {
                        (JsValue::Number(a), JsValue::Number(b)) => JsValue::Number(a * b),
                        _ => JsValue::Number(left_val.to_number() * right_val.to_number()),
                    },
                    "==" => left_val.equals(&right_val),
                    "!=" => left_val.not_equals(&right_val),
                    "===" => left_val.strict_equals(&right_val),
                    "!==" => left_val.strict_not_equals(&right_val),
                    "<" => left_val.less_than(&right_val),
                    ">" => left_val.greater_than(&right_val),
                    "<=" => left_val.less_than_or_equal(&right_val),
                    ">=" => left_val.greater_than_or_equal(&right_val),
                    _ => JsValue::Undefined,
                }
            }
            JsExpression::UnaryOp(op, expr) => {
                let val = expr.evaluate(context);
                match op.as_str() {
                    "-" => JsValue::Number(-val.to_number()),
                    "!" => JsValue::Boolean(!val.to_bool()),
                    _ => JsValue::Undefined,
                }
            }
            JsExpression::LogicalOp(left, op, right) => {
                let left_val = left.evaluate(context);
                match op.as_str() {
                    "&&" => {
                        if left_val.to_bool() {
                            right.evaluate(context)
                        } else {
                            left_val
                        }
                    }
                    "||" => {
                        if left_val.to_bool() {
                            left_val
                        } else {
                            right.evaluate(context)
                        }
                    }
                    _ => JsValue::Undefined,
                }
            }
            JsExpression::Call(func_name, args) => {
                // Evaluate arguments
                let arg_values: Vec<JsValue> =
                    args.iter().map(|arg| arg.evaluate(context)).collect();

                if let Some(value) = builtins::call_global_function(func_name, arg_values.clone()) {
                    return value;
                }

                // Look up function in scope
                if let Some(func_value) = context.scope.borrow().get(func_name) {
                    if let Some(func) = func_value.as_function() {
                        // Execute the function
                        func.execute(
                            &arg_values,
                            Rc::clone(&context.scope),
                            context.this_context.as_ref().map(Rc::clone),
                        )
                    } else {
                        JsValue::Undefined
                    }
                } else {
                    JsValue::Undefined
                }
            }
            JsExpression::MemberAccess(object, member) => {
                let obj_val = object.evaluate(context);
                obj_val.get_property(member)
            }
            JsExpression::IndexAccess(object, index) => {
                let obj_val = object.evaluate(context);
                let index_val = index.evaluate(context);
                obj_val.get_index(&index_val)
            }
            JsExpression::MethodCall {
                object,
                method,
                args,
            } => {
                let obj_val = object.evaluate(context);
                let arg_values: Vec<JsValue> =
                    args.iter().map(|arg| arg.evaluate(context)).collect();

                if let JsExpression::Identifier(obj_name) = object.as_ref() {
                    if let Some(result) =
                        builtins::try_call_named_method(obj_name, method, arg_values.clone())
                    {
                        return result;
                    }
                }
                let (result, updated_obj) =
                    obj_val.execute_method(method, arg_values, Rc::clone(&context.scope));
                if let JsExpression::Identifier(obj_name) = object.as_ref() {
                    context.scope.borrow_mut().set(obj_name, updated_obj);
                }
                result
            }
        }
    }
}

impl JsStatement {
    fn execute_with_flow(&self, context: &JsExecutionContext) -> StatementFlow {
        match self {
            JsStatement::Expression(expr) => StatementFlow::Continue(expr.evaluate(context)),
            JsStatement::Assignment(var_name, expr) => {
                let value = expr.evaluate(context);
                context.scope.borrow_mut().set(var_name, value.clone());
                StatementFlow::Continue(value)
            }
            JsStatement::Declaration { kind, name, expr } => {
                let value = expr.evaluate(context);
                context
                    .scope
                    .borrow_mut()
                    .define(name.clone(), value.clone(), *kind);
                StatementFlow::Continue(value)
            }
            JsStatement::MemberAssignment(target, member, expr) => {
                let value = expr.evaluate(context);

                match target {
                    JsExpression::This => {
                        if let Some(this_ref) = &context.this_context {
                            let mut this_obj = this_ref.borrow_mut();
                            this_obj.set_property(member, value.clone());
                        }
                    }
                    JsExpression::Identifier(obj_name) => {
                        if let Some(mut obj_val) = context.scope.borrow().get(obj_name) {
                            obj_val.set_property(member, value.clone());
                            context.scope.borrow_mut().set(obj_name, obj_val);
                        }
                    }
                    _ => {
                        let mut target_val = target.evaluate(context);
                        target_val.set_property(member, value.clone());
                    }
                }

                StatementFlow::Continue(value)
            }
            JsStatement::IndexAssignment(target, index, expr) => {
                let value = expr.evaluate(context);
                let index_val = index.evaluate(context);
                let key = index_val.to_property_key();
                match target {
                    JsExpression::Identifier(name) => {
                        if let Some(mut obj) = context.scope.borrow().get(name) {
                            obj.set_property(&key, value.clone());
                            context.scope.borrow_mut().set(name, obj);
                        }
                    }
                    _ => {
                        let mut obj = target.evaluate(context);
                        obj.set_property(&key, value.clone());
                    }
                }
                StatementFlow::Continue(value)
            }
            JsStatement::Return(expr_opt) => {
                let value = expr_opt
                    .as_ref()
                    .map(|expr| expr.evaluate(context))
                    .unwrap_or(JsValue::Undefined);
                StatementFlow::Return(value)
            }
            JsStatement::Throw(expr) => StatementFlow::Throw(expr.evaluate(context)),
            JsStatement::IfStatement(condition, body, else_body) => {
                if condition.evaluate(context).to_bool() {
                    let mut result = JsValue::Undefined;
                    for stmt in body {
                        match stmt.execute_with_flow(context) {
                            StatementFlow::Continue(value) => result = value,
                            StatementFlow::Return(value) => return StatementFlow::Return(value),
                            StatementFlow::Throw(value) => return StatementFlow::Throw(value),
                        }
                    }
                    StatementFlow::Continue(result)
                } else {
                    if let Some(else_block) = else_body {
                        let mut result = JsValue::Undefined;
                        for stmt in else_block {
                            match stmt.execute_with_flow(context) {
                                StatementFlow::Continue(value) => result = value,
                                StatementFlow::Return(value) => {
                                    return StatementFlow::Return(value);
                                }
                                StatementFlow::Throw(value) => return StatementFlow::Throw(value),
                            }
                        }
                        StatementFlow::Continue(result)
                    } else {
                        StatementFlow::Continue(JsValue::Undefined)
                    }
                }
            }
            JsStatement::WhileLoop(condition, body) => {
                let mut result = JsValue::Undefined;
                while condition.evaluate(context).to_bool() {
                    for stmt in body {
                        match stmt.execute_with_flow(context) {
                            StatementFlow::Continue(value) => result = value,
                            StatementFlow::Return(value) => return StatementFlow::Return(value),
                            StatementFlow::Throw(value) => return StatementFlow::Throw(value),
                        }
                    }
                }
                StatementFlow::Continue(result)
            }
            JsStatement::DoWhileLoop(body, condition) => {
                let mut result = JsValue::Undefined;
                loop {
                    for stmt in body {
                        match stmt.execute_with_flow(context) {
                            StatementFlow::Continue(value) => result = value,
                            StatementFlow::Return(value) => return StatementFlow::Return(value),
                            StatementFlow::Throw(value) => return StatementFlow::Throw(value),
                        }
                    }
                    if !condition.evaluate(context).to_bool() {
                        break;
                    }
                }
                StatementFlow::Continue(result)
            }
            JsStatement::ForLoop {
                init,
                condition,
                increment,
                body,
            } => {
                if let Some(init_stmt) = init {
                    match init_stmt.execute_with_flow(context) {
                        StatementFlow::Continue(_) => {}
                        StatementFlow::Return(value) => return StatementFlow::Return(value),
                        StatementFlow::Throw(value) => return StatementFlow::Throw(value),
                    }
                }

                let mut result = JsValue::Undefined;
                while condition
                    .as_ref()
                    .map(|c| c.evaluate(context).to_bool())
                    .unwrap_or(true)
                {
                    for stmt in body {
                        match stmt.execute_with_flow(context) {
                            StatementFlow::Continue(value) => result = value,
                            StatementFlow::Return(value) => return StatementFlow::Return(value),
                            StatementFlow::Throw(value) => return StatementFlow::Throw(value),
                        }
                    }

                    if let Some(incr_stmt) = increment {
                        match incr_stmt.execute_with_flow(context) {
                            StatementFlow::Continue(_) => {}
                            StatementFlow::Return(value) => return StatementFlow::Return(value),
                            StatementFlow::Throw(value) => return StatementFlow::Throw(value),
                        }
                    }
                }
                StatementFlow::Continue(result)
            }
            JsStatement::TryCatchFinally {
                try_block,
                catch_param,
                catch_block,
                finally_block,
            } => {
                let mut flow = StatementFlow::Continue(JsValue::Undefined);
                for stmt in try_block {
                    flow = stmt.execute_with_flow(context);
                    if !matches!(flow, StatementFlow::Continue(_)) {
                        break;
                    }
                }

                if let StatementFlow::Throw(thrown) = flow {
                    if let Some(catch_stmts) = catch_block {
                        let catch_scope = JsScope::new_child(Rc::clone(&context.scope));
                        if let Some(param) = catch_param {
                            catch_scope.borrow_mut().define(
                                param.clone(),
                                thrown,
                                JsDeclarationKind::Let,
                            );
                        }
                        let catch_ctx = JsExecutionContext {
                            scope: catch_scope,
                            this_context: context.this_context.as_ref().map(Rc::clone),
                        };
                        let mut catch_flow = StatementFlow::Continue(JsValue::Undefined);
                        for stmt in catch_stmts {
                            catch_flow = stmt.execute_with_flow(&catch_ctx);
                            if !matches!(catch_flow, StatementFlow::Continue(_)) {
                                break;
                            }
                        }
                        flow = catch_flow;
                    } else {
                        flow = StatementFlow::Throw(thrown);
                    }
                }

                if let Some(finally_stmts) = finally_block {
                    let mut finally_flow = StatementFlow::Continue(JsValue::Undefined);
                    for stmt in finally_stmts {
                        finally_flow = stmt.execute_with_flow(context);
                        if !matches!(finally_flow, StatementFlow::Continue(_)) {
                            break;
                        }
                    }
                    if !matches!(finally_flow, StatementFlow::Continue(_)) {
                        return finally_flow;
                    }
                }

                flow
            }
        }
    }

    pub fn execute(&self, context: &JsExecutionContext) -> JsValue {
        match self.execute_with_flow(context) {
            StatementFlow::Continue(value)
            | StatementFlow::Return(value)
            | StatementFlow::Throw(value) => value,
        }
    }
}

impl JsValue {
    pub fn new_error(kind: JsErrorKind, message: impl Into<String>) -> JsValue {
        JsValue::Error(JsError {
            kind,
            message: message.into(),
            stack: None,
            properties: HashMap::new(),
        })
    }

    pub fn new_type_error(message: impl Into<String>) -> JsValue {
        Self::new_error(JsErrorKind::TypeError, message)
    }

    /// Converts a JsValue to a primitive string representation.
    /// This is used for operations like addition where coercion is needed.
    pub fn to_primitive_string(&self) -> String {
        match self {
            JsValue::String(s) => s.clone(),
            JsValue::Number(n) => n.to_string(),
            JsValue::Boolean(b) => b.to_string(),
            JsValue::Null => "null".to_string(),
            JsValue::Undefined => "undefined".to_string(),
            JsValue::Array(arr) => arr
                .iter()
                .map(|v| v.to_primitive_string())
                .collect::<Vec<_>>()
                .join(","),
            JsValue::Object(_) => "[object Object]".to_string(),
            JsValue::Function(func) => {
                format!(
                    "function({}) {{ [native code] }}",
                    func.parameters.join(", ")
                )
            }
            JsValue::Error(err) => {
                if err.message.is_empty() {
                    err.kind.as_str().to_string()
                } else {
                    format!("{}: {}", err.kind.as_str(), err.message)
                }
            }
        }
    }

    /// Converts a JsValue to the string format expected by `console.log`.
    /// Strings are not quoted, but other values are formatted as with Display.
    pub fn to_console_string(&self) -> String {
        match self {
            JsValue::String(s) => s.clone(),
            JsValue::Function(_) => self.to_primitive_string(),
            _ => self.to_string(),
        }
    }

    /// Implements JavaScript's addition operator.
    /// Handles numeric addition and string concatenation.
    pub fn add(&self, other: &JsValue) -> JsValue {
        if matches!(self, JsValue::String(_)) || matches!(other, JsValue::String(_)) {
            let s1 = self.to_primitive_string();
            let s2 = other.to_primitive_string();
            return JsValue::String(s1 + &s2);
        }

        JsValue::Number(self.to_number() + other.to_number())
    }

    pub fn subtract(&self, other: &JsValue) -> JsValue {
        JsValue::Number(self.to_number() - other.to_number())
    }

    /// Pushes a value to the end of a `JsValue::Array`.
    /// Returns the new length of the array.
    /// If the value is not an array, returns `Undefined`.
    pub fn push(&mut self, value: JsValue) -> JsValue {
        if let JsValue::Array(arr) = self {
            arr.push(value);
            JsValue::Number(arr.len() as f64)
        } else {
            JsValue::Undefined
        }
    }

    /// Removes and returns the last element of a `JsValue::Array`.
    /// If the array is empty or the value is not an array, returns `Undefined`.
    pub fn pop(&mut self) -> JsValue {
        if let JsValue::Array(arr) = self {
            arr.pop().unwrap_or(JsValue::Undefined)
        } else {
            JsValue::Undefined
        }
    }

    /// Implements JavaScript's abstract equality comparison (`==`).
    pub fn equals(&self, other: &JsValue) -> JsValue {
        let result = match (self, other) {
            // Strict equality cases
            (JsValue::Number(a), JsValue::Number(b)) => a == b,
            (JsValue::String(a), JsValue::String(b)) => a == b,
            (JsValue::Boolean(a), JsValue::Boolean(b)) => a == b,
            (JsValue::Undefined, JsValue::Undefined) | (JsValue::Null, JsValue::Null) => true,
            // Coercion cases
            (JsValue::Null, JsValue::Undefined) | (JsValue::Undefined, JsValue::Null) => true,
            (JsValue::Number(_), JsValue::String(_)) | (JsValue::String(_), JsValue::Number(_)) => {
                self.to_number() == other.to_number()
            }
            (JsValue::Boolean(_), _) => self.to_number() == other.to_number(),
            (_, JsValue::Boolean(_)) => self.to_number() == other.to_number(),
            _ => false, // Default for objects, arrays, functions
        };
        JsValue::Boolean(result)
    }

    /// Implements JavaScript's inequality comparison (`!=`).
    pub fn not_equals(&self, other: &JsValue) -> JsValue {
        if let JsValue::Boolean(b) = self.equals(other) {
            JsValue::Boolean(!b)
        } else {
            // Should be unreachable
            JsValue::Boolean(true)
        }
    }

    pub fn strict_equals(&self, other: &JsValue) -> JsValue {
        let result = match (self, other) {
            (JsValue::Number(a), JsValue::Number(b)) => a == b,
            (JsValue::String(a), JsValue::String(b)) => a == b,
            (JsValue::Boolean(a), JsValue::Boolean(b)) => a == b,
            (JsValue::Null, JsValue::Null) => true,
            (JsValue::Undefined, JsValue::Undefined) => true,
            _ => false,
        };
        JsValue::Boolean(result)
    }

    pub fn strict_not_equals(&self, other: &JsValue) -> JsValue {
        if let JsValue::Boolean(b) = self.strict_equals(other) {
            JsValue::Boolean(!b)
        } else {
            JsValue::Boolean(true)
        }
    }

    /// Helper for relational comparisons, handles JS string vs number logic.
    fn compare_relational(&self, other: &JsValue) -> Option<std::cmp::Ordering> {
        if let (JsValue::String(s1), JsValue::String(s2)) = (self, other) {
            s1.partial_cmp(s2)
        } else {
            self.to_number().partial_cmp(&other.to_number())
        }
    }

    pub fn less_than(&self, other: &JsValue) -> JsValue {
        JsValue::Boolean(matches!(
            self.compare_relational(other),
            Some(std::cmp::Ordering::Less)
        ))
    }

    pub fn greater_than(&self, other: &JsValue) -> JsValue {
        JsValue::Boolean(matches!(
            self.compare_relational(other),
            Some(std::cmp::Ordering::Greater)
        ))
    }

    pub fn less_than_or_equal(&self, other: &JsValue) -> JsValue {
        JsValue::Boolean(matches!(
            self.compare_relational(other),
            Some(std::cmp::Ordering::Less | std::cmp::Ordering::Equal)
        ))
    }

    pub fn greater_than_or_equal(&self, other: &JsValue) -> JsValue {
        JsValue::Boolean(matches!(
            self.compare_relational(other),
            Some(std::cmp::Ordering::Greater | std::cmp::Ordering::Equal)
        ))
    }

    pub fn to_bool(&self) -> bool {
        match self {
            JsValue::Boolean(b) => *b,
            JsValue::Number(n) => *n != 0.0 && !n.is_nan(),
            JsValue::String(s) => !s.is_empty(),
            JsValue::Null | JsValue::Undefined => false,
            JsValue::Object(_) | JsValue::Array(_) | JsValue::Function(_) | JsValue::Error(_) => {
                true
            }
        }
    }

    pub fn to_number(&self) -> f64 {
        match self {
            JsValue::Number(n) => *n,
            JsValue::String(s) => s.parse::<f64>().unwrap_or(f64::NAN),
            JsValue::Boolean(true) => 1.0,
            JsValue::Boolean(false) => 0.0,
            JsValue::Null => 0.0,
            JsValue::Undefined => f64::NAN,
            JsValue::Error(_) => f64::NAN,
            _ => f64::NAN,
        }
    }

    /// Sets a property on an object, returns the value that was set
    pub fn set_property(&mut self, key: &str, value: JsValue) -> JsValue {
        if let JsValue::Object(map) = self {
            map.insert(key.to_string(), value.clone());
            value
        } else if let JsValue::Error(err) = self {
            match key {
                "name" => {
                    if let JsValue::String(name) = &value {
                        err.kind = if name == "TypeError" {
                            JsErrorKind::TypeError
                        } else {
                            JsErrorKind::Error
                        };
                    } else {
                        err.properties.insert(key.to_string(), value.clone());
                    }
                    value
                }
                "message" => {
                    err.message = value.to_primitive_string();
                    value
                }
                "stack" => {
                    err.stack = Some(value.to_primitive_string());
                    value
                }
                _ => {
                    err.properties.insert(key.to_string(), value.clone());
                    value
                }
            }
        } else {
            JsValue::Undefined
        }
    }

    /// Gets a property from an object
    pub fn get_property(&self, key: &str) -> JsValue {
        match self {
            JsValue::Object(map) => map.get(key).unwrap_or(&JsValue::Undefined).clone(),
            JsValue::Error(err) => match key {
                "name" => JsValue::String(err.kind.as_str().to_string()),
                "message" => JsValue::String(err.message.clone()),
                "stack" => err
                    .stack
                    .as_ref()
                    .map(|s| JsValue::String(s.clone()))
                    .unwrap_or(JsValue::Undefined),
                _ => err
                    .properties
                    .get(key)
                    .cloned()
                    .unwrap_or(JsValue::Undefined),
            },
            _ => JsValue::Undefined,
        }
    }

    pub fn to_property_key(&self) -> String {
        match self {
            JsValue::String(s) => s.clone(),
            JsValue::Number(n) if n.fract() == 0.0 => format!("{}", *n as i64),
            JsValue::Number(n) => n.to_string(),
            JsValue::Boolean(b) => b.to_string(),
            JsValue::Null => "null".to_string(),
            JsValue::Undefined => "undefined".to_string(),
            _ => self.to_primitive_string(),
        }
    }

    pub fn get_index(&self, index: &JsValue) -> JsValue {
        match self {
            JsValue::Array(arr) => {
                let idx = index.to_number() as isize;
                if idx >= 0 {
                    arr.get(idx as usize).cloned().unwrap_or(JsValue::Undefined)
                } else {
                    JsValue::Undefined
                }
            }
            JsValue::Object(_) | JsValue::Error(_) => self.get_property(&index.to_property_key()),
            _ => JsValue::Undefined,
        }
    }

    pub fn set_index(&mut self, index: &JsValue, value: JsValue) {
        match self {
            JsValue::Array(arr) => {
                let idx = index.to_number() as isize;
                if idx >= 0 {
                    let idx = idx as usize;
                    if idx < arr.len() {
                        arr[idx] = value;
                    } else if idx == arr.len() {
                        arr.push(value);
                    }
                }
            }
            JsValue::Object(map) => {
                map.insert(index.to_property_key(), value);
            }
            JsValue::Error(err) => {
                err.properties.insert(index.to_property_key(), value);
            }
            _ => {}
        }
    }

    pub fn as_function(&self) -> Option<&JsFunction> {
        match self {
            JsValue::Function(func) => Some(func),
            _ => None,
        }
    }

    pub fn execute_method(
        &self,
        method_name: &str,
        args: Vec<JsValue>,
        global_scope: Rc<RefCell<JsScope>>,
    ) -> (JsValue, JsValue) {
        builtins::call_method(self, method_name, args, global_scope)
    }
}

impl From<f64> for JsValue {
    fn from(num: f64) -> Self {
        JsValue::Number(num)
    }
}

impl From<String> for JsValue {
    fn from(s: String) -> Self {
        JsValue::String(s)
    }
}

impl From<&str> for JsValue {
    fn from(s: &str) -> Self {
        JsValue::String(s.to_string())
    }
}

impl From<bool> for JsValue {
    fn from(b: bool) -> Self {
        JsValue::Boolean(b)
    }
}

impl fmt::Display for JsValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JsValue::Number(n) => write!(f, "{}", n),
            JsValue::String(s) => write!(f, "\"{}\"", s),
            JsValue::Boolean(b) => write!(f, "{}", b),
            JsValue::Null => write!(f, "null"),
            JsValue::Undefined => write!(f, "undefined"),
            JsValue::Object(_) => write!(f, "[object Object]"),
            JsValue::Array(arr) => {
                let elems: Vec<String> = arr.iter().map(|v| v.to_string()).collect();
                write!(f, "[{}]", elems.join(", "))
            }
            JsValue::Function(func) => {
                write!(f, "function({}) {{...}}", func.parameters.join(", "))
            }
            JsValue::Error(err) => {
                if err.message.is_empty() {
                    write!(f, "{}", err.kind.as_str())
                } else {
                    write!(f, "{}: {}", err.kind.as_str(), err.message)
                }
            }
        }
    }
}

/// Logs a sequence of JsValue instances to the console, similar to JavaScript's `console.log`.
pub fn console_log(values: Vec<JsValue>) {
    let output = values
        .iter()
        .map(|v| v.to_console_string())
        .collect::<Vec<String>>()
        .join(" ");
    println!("{}", output);
}

fn compile_program(statements: &[JsStatement]) -> BytecodeProgram {
    let mut instructions = Vec::new();
    for statement in statements {
        compile_statement(statement, &mut instructions);
    }
    BytecodeProgram { instructions }
}

fn compile_statement(statement: &JsStatement, out: &mut Vec<Instruction>) {
    match statement {
        JsStatement::Expression(expr) => {
            compile_expression(expr, out);
            out.push(Instruction::SetLast);
            out.push(Instruction::Pop);
        }
        JsStatement::Assignment(name, expr) => {
            compile_expression(expr, out);
            out.push(Instruction::Dup);
            out.push(Instruction::StoreIdentifier(name.clone()));
            out.push(Instruction::SetLast);
            out.push(Instruction::Pop);
        }
        JsStatement::Declaration { kind, name, expr } => {
            compile_expression(expr, out);
            out.push(Instruction::Dup);
            out.push(Instruction::Declare {
                kind: *kind,
                name: name.clone(),
            });
            out.push(Instruction::SetLast);
            out.push(Instruction::Pop);
        }
        JsStatement::MemberAssignment(target, member, expr) => {
            match target {
                JsExpression::This => {
                    compile_expression(expr, out);
                    out.push(Instruction::AssignThisMember(member.clone()));
                }
                JsExpression::Identifier(name) => {
                    compile_expression(expr, out);
                    out.push(Instruction::AssignNamedMember {
                        object_name: name.clone(),
                        member: member.clone(),
                    });
                }
                _ => {
                    compile_expression(target, out);
                    compile_expression(expr, out);
                    out.push(Instruction::AssignComputedMember(member.clone()));
                }
            }
            out.push(Instruction::SetLast);
            out.push(Instruction::Pop);
        }
        JsStatement::IndexAssignment(target, index, expr) => {
            compile_expression(index, out);
            compile_expression(expr, out);
            if let JsExpression::Identifier(name) = target {
                out.push(Instruction::AssignNamedIndex(name.clone()));
            } else {
                compile_expression(target, out);
                out.push(Instruction::AssignComputedIndex);
            }
            out.push(Instruction::SetLast);
            out.push(Instruction::Pop);
        }
        JsStatement::Return(expr_opt) => {
            if let Some(expr) = expr_opt {
                compile_expression(expr, out);
            } else {
                out.push(Instruction::PushUndefined);
            }
            out.push(Instruction::Return);
        }
        JsStatement::Throw(expr) => {
            compile_expression(expr, out);
            out.push(Instruction::Throw);
        }
        JsStatement::IfStatement(condition, body, else_body) => {
            compile_expression(condition, out);
            let jmp_false_idx = out.len();
            out.push(Instruction::JumpIfFalse(usize::MAX));
            for stmt in body {
                compile_statement(stmt, out);
            }
            if let Some(else_block) = else_body {
                let jmp_end_idx = out.len();
                out.push(Instruction::Jump(usize::MAX));
                let else_start = out.len();
                out[jmp_false_idx] = Instruction::JumpIfFalse(else_start);
                for stmt in else_block {
                    compile_statement(stmt, out);
                }
                let end = out.len();
                out[jmp_end_idx] = Instruction::Jump(end);
            } else {
                let end = out.len();
                out[jmp_false_idx] = Instruction::JumpIfFalse(end);
            }
        }
        JsStatement::WhileLoop(condition, body) => {
            let loop_start = out.len();
            compile_expression(condition, out);
            let jmp_false_idx = out.len();
            out.push(Instruction::JumpIfFalse(usize::MAX));
            for stmt in body {
                compile_statement(stmt, out);
            }
            out.push(Instruction::Jump(loop_start));
            let end = out.len();
            out[jmp_false_idx] = Instruction::JumpIfFalse(end);
        }
        JsStatement::DoWhileLoop(body, condition) => {
            let loop_start = out.len();
            for stmt in body {
                compile_statement(stmt, out);
            }
            compile_expression(condition, out);
            let jmp_false_idx = out.len();
            out.push(Instruction::JumpIfFalse(usize::MAX));
            out.push(Instruction::Jump(loop_start));
            let end = out.len();
            out[jmp_false_idx] = Instruction::JumpIfFalse(end);
        }
        JsStatement::ForLoop {
            init,
            condition,
            increment,
            body,
        } => {
            if let Some(init_stmt) = init {
                compile_statement(init_stmt, out);
            }
            let loop_start = out.len();
            if let Some(cond) = condition {
                compile_expression(cond, out);
            } else {
                out.push(Instruction::PushLiteral(JsValue::Boolean(true)));
            }
            let jmp_false_idx = out.len();
            out.push(Instruction::JumpIfFalse(usize::MAX));
            for stmt in body {
                compile_statement(stmt, out);
            }
            if let Some(incr_stmt) = increment {
                compile_statement(incr_stmt, out);
            }
            out.push(Instruction::Jump(loop_start));
            let end = out.len();
            out[jmp_false_idx] = Instruction::JumpIfFalse(end);
        }
        JsStatement::TryCatchFinally {
            try_block,
            catch_param,
            catch_block,
            finally_block,
        } => {
            let try_program = compile_program(try_block);
            let catch_program = catch_block.as_ref().map(|b| compile_program(b));
            let finally_program = finally_block.as_ref().map(|b| compile_program(b));
            out.push(Instruction::TryCatchFinally {
                try_program,
                catch_param: catch_param.clone(),
                catch_program,
                finally_program,
            });
            out.push(Instruction::SetLast);
            out.push(Instruction::Pop);
        }
    }
}

fn compile_expression(expr: &JsExpression, out: &mut Vec<Instruction>) {
    match expr {
        JsExpression::Literal(v) => out.push(Instruction::PushLiteral(v.clone())),
        JsExpression::Identifier(name) => out.push(Instruction::LoadIdentifier(name.clone())),
        JsExpression::This => out.push(Instruction::LoadThis),
        JsExpression::BinaryOp(left, op, right) => {
            compile_expression(left, out);
            compile_expression(right, out);
            out.push(Instruction::BinaryOp(op.clone()));
        }
        JsExpression::UnaryOp(op, expr) => {
            compile_expression(expr, out);
            out.push(Instruction::UnaryOp(op.clone()));
        }
        JsExpression::LogicalOp(left, op, right) => match op.as_str() {
            "&&" => {
                compile_expression(left, out);
                out.push(Instruction::Dup);
                let jump_idx = out.len();
                out.push(Instruction::LogicalAnd(usize::MAX));
                out.push(Instruction::Pop);
                compile_expression(right, out);
                let end = out.len();
                out[jump_idx] = Instruction::LogicalAnd(end);
            }
            "||" => {
                compile_expression(left, out);
                out.push(Instruction::Dup);
                let jump_idx = out.len();
                out.push(Instruction::LogicalOr(usize::MAX));
                out.push(Instruction::Pop);
                compile_expression(right, out);
                let end = out.len();
                out[jump_idx] = Instruction::LogicalOr(end);
            }
            _ => out.push(Instruction::PushUndefined),
        },
        JsExpression::Call(name, args) => {
            for arg in args {
                compile_expression(arg, out);
            }
            out.push(Instruction::Call(name.clone(), args.len()));
        }
        JsExpression::MemberAccess(object, member) => {
            compile_expression(object, out);
            out.push(Instruction::MemberAccess(member.clone()));
        }
        JsExpression::IndexAccess(object, index) => {
            compile_expression(object, out);
            compile_expression(index, out);
            out.push(Instruction::IndexAccess);
        }
        JsExpression::MethodCall {
            object,
            method,
            args,
        } => {
            for arg in args {
                compile_expression(arg, out);
            }
            if let JsExpression::Identifier(name) = object.as_ref() {
                out.push(Instruction::MethodCallNamed {
                    object_name: name.clone(),
                    method: method.clone(),
                    arg_count: args.len(),
                });
            } else {
                compile_expression(object, out);
                out.push(Instruction::MethodCall {
                    method: method.clone(),
                    arg_count: args.len(),
                });
            }
        }
    }
}

fn pop_args(stack: &mut Vec<JsValue>, count: usize) -> Vec<JsValue> {
    let mut args = Vec::with_capacity(count);
    for _ in 0..count {
        args.push(stack.pop().unwrap_or(JsValue::Undefined));
    }
    args.reverse();
    args
}

fn execute_bytecode(program: &BytecodeProgram, context: &JsExecutionContext) -> JsValue {
    match execute_bytecode_flow(program, context) {
        VmFlow::Continue(value) | VmFlow::Return(value) | VmFlow::Throw(value) => value,
    }
}

fn execute_bytecode_flow(program: &BytecodeProgram, context: &JsExecutionContext) -> VmFlow {
    let mut stack = Vec::<JsValue>::new();
    let mut ip = 0usize;
    let mut last = JsValue::Undefined;

    while ip < program.instructions.len() {
        match &program.instructions[ip] {
            Instruction::PushLiteral(value) => stack.push(value.clone()),
            Instruction::PushUndefined => stack.push(JsValue::Undefined),
            Instruction::LoadIdentifier(name) => {
                if let Some(value) = builtins::resolve_global(name) {
                    stack.push(value);
                } else {
                    stack.push(
                        context
                            .scope
                            .borrow()
                            .get(name)
                            .unwrap_or(JsValue::Undefined),
                    );
                }
            }
            Instruction::LoadThis => stack.push(
                context
                    .this_context
                    .as_ref()
                    .map(|t| t.borrow().clone())
                    .unwrap_or(JsValue::Undefined),
            ),
            Instruction::StoreIdentifier(name) => {
                let value = stack.pop().unwrap_or(JsValue::Undefined);
                context.scope.borrow_mut().set(name, value);
            }
            Instruction::Declare { kind, name } => {
                let value = stack.pop().unwrap_or(JsValue::Undefined);
                context
                    .scope
                    .borrow_mut()
                    .define(name.clone(), value, *kind);
            }
            Instruction::Dup => {
                if let Some(top) = stack.last().cloned() {
                    stack.push(top);
                }
            }
            Instruction::Pop => {
                let _ = stack.pop();
            }
            Instruction::SetLast => {
                last = stack.last().cloned().unwrap_or(JsValue::Undefined);
            }
            Instruction::BinaryOp(op) => {
                let right = stack.pop().unwrap_or(JsValue::Undefined);
                let left = stack.pop().unwrap_or(JsValue::Undefined);
                let value = match op.as_str() {
                    "+" => left.add(&right),
                    "-" => JsValue::Number(left.to_number() - right.to_number()),
                    "*" => JsValue::Number(left.to_number() * right.to_number()),
                    "==" => left.equals(&right),
                    "!=" => left.not_equals(&right),
                    "===" => left.strict_equals(&right),
                    "!==" => left.strict_not_equals(&right),
                    "<" => left.less_than(&right),
                    ">" => left.greater_than(&right),
                    "<=" => left.less_than_or_equal(&right),
                    ">=" => left.greater_than_or_equal(&right),
                    _ => JsValue::Undefined,
                };
                stack.push(value);
            }
            Instruction::UnaryOp(op) => {
                let value = stack.pop().unwrap_or(JsValue::Undefined);
                let result = match op.as_str() {
                    "-" => JsValue::Number(-value.to_number()),
                    "!" => JsValue::Boolean(!value.to_bool()),
                    _ => JsValue::Undefined,
                };
                stack.push(result);
            }
            Instruction::LogicalAnd(target) => {
                let top = stack.last().cloned().unwrap_or(JsValue::Undefined);
                if !top.to_bool() {
                    ip = *target;
                    continue;
                }
            }
            Instruction::LogicalOr(target) => {
                let top = stack.last().cloned().unwrap_or(JsValue::Undefined);
                if top.to_bool() {
                    ip = *target;
                    continue;
                }
            }
            Instruction::Call(name, arg_count) => {
                let args = pop_args(&mut stack, *arg_count);
                let result = if let Some(value) = builtins::call_global_function(name, args.clone())
                {
                    value
                } else if let Some(func_value) = context.scope.borrow().get(name) {
                    if let Some(func) = func_value.as_function() {
                        func.execute(
                            &args,
                            Rc::clone(&context.scope),
                            context.this_context.clone(),
                        )
                    } else {
                        JsValue::Undefined
                    }
                } else {
                    JsValue::Undefined
                };
                stack.push(result);
            }
            Instruction::MemberAccess(member) => {
                let obj = stack.pop().unwrap_or(JsValue::Undefined);
                stack.push(obj.get_property(member));
            }
            Instruction::IndexAccess => {
                let index = stack.pop().unwrap_or(JsValue::Undefined);
                let obj = stack.pop().unwrap_or(JsValue::Undefined);
                stack.push(obj.get_index(&index));
            }
            Instruction::MethodCall { method, arg_count } => {
                let obj = stack.pop().unwrap_or(JsValue::Undefined);
                let args = pop_args(&mut stack, *arg_count);
                let (result, _) = obj.execute_method(method, args, Rc::clone(&context.scope));
                stack.push(result);
            }
            Instruction::MethodCallNamed {
                object_name,
                method,
                arg_count,
            } => {
                let args = pop_args(&mut stack, *arg_count);
                if let Some(result) =
                    builtins::try_call_named_method(object_name, method, args.clone())
                {
                    stack.push(result);
                } else {
                    let obj = context
                        .scope
                        .borrow()
                        .get(object_name)
                        .unwrap_or(JsValue::Undefined);
                    let (result, updated_obj) =
                        obj.execute_method(method, args, Rc::clone(&context.scope));
                    context.scope.borrow_mut().set(object_name, updated_obj);
                    stack.push(result);
                }
            }
            Instruction::AssignThisMember(member) => {
                let value = stack.pop().unwrap_or(JsValue::Undefined);
                if let Some(this_ref) = &context.this_context {
                    this_ref.borrow_mut().set_property(member, value.clone());
                }
                stack.push(value);
            }
            Instruction::AssignNamedMember {
                object_name,
                member,
            } => {
                let value = stack.pop().unwrap_or(JsValue::Undefined);
                if let Some(mut obj) = context.scope.borrow().get(object_name) {
                    obj.set_property(member, value.clone());
                    context.scope.borrow_mut().set(object_name, obj);
                }
                stack.push(value);
            }
            Instruction::AssignComputedMember(member) => {
                let value = stack.pop().unwrap_or(JsValue::Undefined);
                let mut target = stack.pop().unwrap_or(JsValue::Undefined);
                target.set_property(member, value.clone());
                stack.push(value);
            }
            Instruction::AssignNamedIndex(name) => {
                let value = stack.pop().unwrap_or(JsValue::Undefined);
                let index = stack.pop().unwrap_or(JsValue::Undefined);
                if let Some(mut obj) = context.scope.borrow().get(name) {
                    obj.set_index(&index, value.clone());
                    context.scope.borrow_mut().set(name, obj);
                }
                stack.push(value);
            }
            Instruction::AssignComputedIndex => {
                let mut obj = stack.pop().unwrap_or(JsValue::Undefined);
                let value = stack.pop().unwrap_or(JsValue::Undefined);
                let index = stack.pop().unwrap_or(JsValue::Undefined);
                obj.set_index(&index, value.clone());
                stack.push(value);
            }
            Instruction::JumpIfFalse(target) => {
                let cond = stack.pop().unwrap_or(JsValue::Undefined);
                if !cond.to_bool() {
                    ip = *target;
                    continue;
                }
            }
            Instruction::Jump(target) => {
                ip = *target;
                continue;
            }
            Instruction::Throw => {
                return VmFlow::Throw(stack.pop().unwrap_or(JsValue::Undefined));
            }
            Instruction::TryCatchFinally {
                try_program,
                catch_param,
                catch_program,
                finally_program,
            } => {
                let mut flow = execute_bytecode_flow(try_program, context);

                if let VmFlow::Throw(thrown) = flow {
                    if let Some(catch_prog) = catch_program {
                        let catch_scope = JsScope::new_child(Rc::clone(&context.scope));
                        if let Some(param) = catch_param {
                            catch_scope.borrow_mut().define(
                                param.clone(),
                                thrown,
                                JsDeclarationKind::Let,
                            );
                        }
                        let catch_ctx = JsExecutionContext {
                            scope: catch_scope,
                            this_context: context.this_context.as_ref().map(Rc::clone),
                        };
                        flow = execute_bytecode_flow(catch_prog, &catch_ctx);
                    } else {
                        flow = VmFlow::Throw(thrown);
                    }
                }

                if let Some(finally_prog) = finally_program {
                    let finally_flow = execute_bytecode_flow(finally_prog, context);
                    if !matches!(finally_flow, VmFlow::Continue(_)) {
                        return finally_flow;
                    }
                }

                match flow {
                    VmFlow::Continue(value) => {
                        stack.push(value);
                    }
                    VmFlow::Return(value) => return VmFlow::Return(value),
                    VmFlow::Throw(value) => return VmFlow::Throw(value),
                }
            }
            Instruction::Return => {
                return VmFlow::Return(stack.pop().unwrap_or(last));
            }
        }
        ip += 1;
    }

    VmFlow::Continue(last)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn root_context() -> JsExecutionContext {
        JsExecutionContext {
            scope: JsScope::new_root(),
            this_context: None,
        }
    }

    #[test]
    fn return_short_circuits_function_body() {
        let func = JsFunction::new(
            vec![],
            vec![
                JsStatement::Return(Some(JsExpression::Literal(JsValue::Number(1.0)))),
                JsStatement::Return(Some(JsExpression::Literal(JsValue::Number(2.0)))),
            ],
            "f".to_string(),
        );
        let result = func.execute(&[], JsScope::new_root(), None);
        assert!(matches!(result, JsValue::Number(n) if (n - 1.0).abs() < f64::EPSILON));
    }

    #[test]
    fn subtraction_operator_works() {
        let context = root_context();
        let expr = JsExpression::BinaryOp(
            Box::new(JsExpression::Literal(JsValue::Number(10.0))),
            "-".to_string(),
            Box::new(JsExpression::Literal(JsValue::Number(3.0))),
        );
        let result = expr.evaluate(&context);
        assert!(matches!(result, JsValue::Number(n) if (n - 7.0).abs() < f64::EPSILON));
    }

    #[test]
    fn relational_nan_follows_js_behavior() {
        let nan = JsValue::Undefined;
        let one = JsValue::Number(1.0);
        assert!(matches!(
            nan.less_than_or_equal(&one),
            JsValue::Boolean(false)
        ));
        assert!(matches!(
            nan.greater_than_or_equal(&one),
            JsValue::Boolean(false)
        ));
    }

    #[test]
    fn program_executes_statements() {
        let context = root_context();
        let program = JsProgram::new(vec![
            JsStatement::Declaration {
                kind: JsDeclarationKind::Let,
                name: "x".to_string(),
                expr: JsExpression::Literal(JsValue::Number(2.0)),
            },
            JsStatement::Expression(JsExpression::BinaryOp(
                Box::new(JsExpression::Identifier("x".to_string())),
                "+".to_string(),
                Box::new(JsExpression::Literal(JsValue::Number(3.0))),
            )),
        ]);
        let out = program.execute(&context);
        assert!(matches!(out, JsValue::Number(n) if (n - 5.0).abs() < f64::EPSILON));
    }

    #[test]
    fn child_scope_reads_parent_and_writes_existing_binding() {
        let root = JsScope::new_root();
        root.borrow_mut().define(
            "x".to_string(),
            JsValue::Number(1.0),
            JsDeclarationKind::Let,
        );
        let child = JsScope::new_child(Rc::clone(&root));

        assert!(
            matches!(child.borrow().get("x"), Some(JsValue::Number(n)) if (n - 1.0).abs() < f64::EPSILON)
        );
        child.borrow_mut().set("x", JsValue::Number(7.0));
        assert!(
            matches!(root.borrow().get("x"), Some(JsValue::Number(n)) if (n - 7.0).abs() < f64::EPSILON)
        );
    }

    #[test]
    fn const_binding_is_not_mutated_by_assignment() {
        let root = JsScope::new_root();
        root.borrow_mut().define(
            "x".to_string(),
            JsValue::Number(1.0),
            JsDeclarationKind::Const,
        );
        let updated = root.borrow_mut().set("x", JsValue::Number(2.0));
        assert!(!updated);
        assert!(
            matches!(root.borrow().get("x"), Some(JsValue::Number(n)) if (n - 1.0).abs() < f64::EPSILON)
        );
    }

    #[test]
    fn throw_without_catch_surfaces_value() {
        let context = root_context();
        let program = JsProgram::new(vec![JsStatement::Throw(JsExpression::Literal(
            JsValue::String("boom".to_string()),
        ))]);
        let out = program.execute(&context);
        assert!(matches!(out, JsValue::String(s) if s == "boom"));
    }

    #[test]
    fn try_catch_binds_exception_value() {
        let context = root_context();
        let program = JsProgram::new(vec![JsStatement::TryCatchFinally {
            try_block: vec![JsStatement::Throw(JsExpression::Literal(JsValue::Number(
                7.0,
            )))],
            catch_param: Some("e".to_string()),
            catch_block: Some(vec![JsStatement::Expression(JsExpression::Identifier(
                "e".to_string(),
            ))]),
            finally_block: None,
        }]);
        let out = program.execute(&context);
        assert!(matches!(out, JsValue::Number(n) if (n - 7.0).abs() < f64::EPSILON));
    }

    #[test]
    fn finally_runs_and_overrides_with_return() {
        let context = root_context();
        let program = JsProgram::new(vec![JsStatement::TryCatchFinally {
            try_block: vec![JsStatement::Throw(JsExpression::Literal(JsValue::String(
                "err".to_string(),
            )))],
            catch_param: Some("e".to_string()),
            catch_block: Some(vec![JsStatement::Expression(JsExpression::Identifier(
                "e".to_string(),
            ))]),
            finally_block: Some(vec![JsStatement::Return(Some(JsExpression::Literal(
                JsValue::Number(99.0),
            )))]),
        }]);
        let out = program.execute(&context);
        assert!(matches!(out, JsValue::Number(n) if (n - 99.0).abs() < f64::EPSILON));
    }

    #[test]
    fn error_constructor_sets_name_and_message() {
        let context = root_context();
        let value = JsExpression::Call(
            "Error".to_string(),
            vec![JsExpression::Literal(JsValue::String("boom".to_string()))],
        )
        .evaluate(&context);
        assert!(matches!(
            value.get_property("name"),
            JsValue::String(s) if s == "Error"
        ));
        assert!(matches!(
            value.get_property("message"),
            JsValue::String(s) if s == "boom"
        ));
    }

    #[test]
    fn type_error_constructor_sets_name() {
        let context = root_context();
        let value = JsExpression::Call(
            "TypeError".to_string(),
            vec![JsExpression::Literal(JsValue::String(
                "bad call".to_string(),
            ))],
        )
        .evaluate(&context);
        assert!(matches!(
            value.get_property("name"),
            JsValue::String(s) if s == "TypeError"
        ));
    }

    #[test]
    fn throw_error_is_catchable_with_message() {
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
        let out = program.execute(&context);
        assert!(matches!(out, JsValue::String(s) if s == "kaboom"));
    }
}
