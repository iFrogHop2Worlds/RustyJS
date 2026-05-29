use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::{JsErrorKind, JsFunction, JsScope, JsValue, console_log};

type BuiltinValueFactory = fn() -> JsValue;
type BuiltinCall = fn(Vec<JsValue>) -> JsValue;
type BuiltinMethodCall = fn(Vec<JsValue>) -> JsValue;

struct BuiltinGlobal {
    value: BuiltinValueFactory,
    call: Option<BuiltinCall>,
    methods: HashMap<&'static str, BuiltinMethodCall>,
}

struct BuiltinRegistry {
    globals: HashMap<&'static str, BuiltinGlobal>,
}

impl BuiltinRegistry {
    fn new() -> Self {
        let mut globals = HashMap::new();
        globals.insert(
            "console",
            BuiltinGlobal {
                value: console_object,
                call: None,
                methods: HashMap::from([("log", console_log_method as BuiltinMethodCall)]),
            },
        );
        globals.insert(
            "Error",
            BuiltinGlobal {
                value: error_constructor,
                call: Some(error_call),
                methods: HashMap::new(),
            },
        );
        globals.insert(
            "TypeError",
            BuiltinGlobal {
                value: type_error_constructor,
                call: Some(type_error_call),
                methods: HashMap::new(),
            },
        );
        globals.insert(
            "Math",
            BuiltinGlobal {
                value: || builtin_object(&["abs", "floor", "ceil", "round", "max", "min", "pow"]),
                call: None,
                methods: HashMap::from([
                    ("abs", math_abs as BuiltinMethodCall),
                    ("floor", math_floor as BuiltinMethodCall),
                    ("ceil", math_ceil as BuiltinMethodCall),
                    ("round", math_round as BuiltinMethodCall),
                    ("max", math_max as BuiltinMethodCall),
                    ("min", math_min as BuiltinMethodCall),
                    ("pow", math_pow as BuiltinMethodCall),
                ]),
            },
        );
        globals.insert(
            "JSON",
            BuiltinGlobal {
                value: || builtin_object(&["stringify", "parse"]),
                call: None,
                methods: HashMap::from([
                    ("stringify", json_stringify as BuiltinMethodCall),
                    ("parse", json_parse as BuiltinMethodCall),
                ]),
            },
        );
        globals.insert(
            "Object",
            BuiltinGlobal {
                value: || builtin_object(&["keys", "values", "assign"]),
                call: None,
                methods: HashMap::from([
                    ("keys", object_keys as BuiltinMethodCall),
                    ("values", object_values as BuiltinMethodCall),
                    ("assign", object_assign as BuiltinMethodCall),
                ]),
            },
        );
        globals.insert(
            "String",
            BuiltinGlobal {
                value: string_constructor,
                call: Some(string_call),
                methods: HashMap::new(),
            },
        );
        globals.insert(
            "Number",
            BuiltinGlobal {
                value: number_constructor,
                call: Some(number_call),
                methods: HashMap::new(),
            },
        );
        globals.insert(
            "Boolean",
            BuiltinGlobal {
                value: boolean_constructor,
                call: Some(boolean_call),
                methods: HashMap::new(),
            },
        );
        Self { globals }
    }

    fn global_property(&self, name: &str) -> Option<JsValue> {
        self.globals.get(name).map(|global| (global.value)())
    }

    fn call_global(&self, name: &str, args: Vec<JsValue>) -> Option<JsValue> {
        self.globals
            .get(name)
            .and_then(|global| global.call.map(|call| call(args)))
    }

    fn call_global_method(
        &self,
        object_name: &str,
        method_name: &str,
        args: Vec<JsValue>,
    ) -> Option<JsValue> {
        self.globals
            .get(object_name)
            .and_then(|global| global.methods.get(method_name))
            .map(|method| method(args))
    }
}

pub(crate) fn global_property(name: &str) -> Option<JsValue> {
    BuiltinRegistry::new().global_property(name)
}

pub(crate) fn call_global(name: &str, args: Vec<JsValue>) -> Option<JsValue> {
    BuiltinRegistry::new().call_global(name, args)
}

pub(crate) fn call_global_method(
    object_name: &str,
    method: &str,
    args: Vec<JsValue>,
) -> Option<JsValue> {
    BuiltinRegistry::new().call_global_method(object_name, method, args)
}

fn console_object() -> JsValue {
    let mut obj = HashMap::new();
    obj.insert(
        "log".to_string(),
        JsValue::Function(JsFunction::new(
            vec!["values".to_string()],
            vec![],
            "log".to_string(),
        )),
    );
    JsValue::Object(obj)
}

fn builtin_object(methods: &[&str]) -> JsValue {
    let mut obj = HashMap::new();
    for method in methods {
        obj.insert(
            (*method).to_string(),
            JsValue::Function(JsFunction::new(vec![], vec![], (*method).to_string())),
        );
    }
    JsValue::Object(obj)
}

fn console_log_method(args: Vec<JsValue>) -> JsValue {
    console_log(args);
    JsValue::Undefined
}

fn error_constructor() -> JsValue {
    JsValue::Function(JsFunction::new(
        vec!["message".to_string()],
        vec![],
        "Error".to_string(),
    ))
}

fn type_error_constructor() -> JsValue {
    JsValue::Function(JsFunction::new(
        vec!["message".to_string()],
        vec![],
        "TypeError".to_string(),
    ))
}

fn error_call(args: Vec<JsValue>) -> JsValue {
    let message = args
        .first()
        .map(|v| v.to_primitive_string())
        .unwrap_or_default();
    JsValue::new_error(JsErrorKind::Error, message)
}

fn type_error_call(args: Vec<JsValue>) -> JsValue {
    let message = args
        .first()
        .map(|v| v.to_primitive_string())
        .unwrap_or_default();
    JsValue::new_type_error(message)
}

fn string_constructor() -> JsValue {
    JsValue::Function(JsFunction::new(
        vec!["value".to_string()],
        vec![],
        "String".to_string(),
    ))
}

fn number_constructor() -> JsValue {
    JsValue::Function(JsFunction::new(
        vec!["value".to_string()],
        vec![],
        "Number".to_string(),
    ))
}

fn boolean_constructor() -> JsValue {
    JsValue::Function(JsFunction::new(
        vec!["value".to_string()],
        vec![],
        "Boolean".to_string(),
    ))
}

fn string_call(args: Vec<JsValue>) -> JsValue {
    JsValue::String(
        args.first()
            .map(JsValue::to_primitive_string)
            .unwrap_or_default(),
    )
}

fn number_call(args: Vec<JsValue>) -> JsValue {
    JsValue::Number(args.first().map(JsValue::to_number).unwrap_or(0.0))
}

fn boolean_call(args: Vec<JsValue>) -> JsValue {
    JsValue::Boolean(args.first().map(JsValue::to_bool).unwrap_or(false))
}

fn first_number(args: &[JsValue]) -> f64 {
    args.first().map(JsValue::to_number).unwrap_or(f64::NAN)
}

fn math_abs(args: Vec<JsValue>) -> JsValue {
    JsValue::Number(first_number(&args).abs())
}

fn math_floor(args: Vec<JsValue>) -> JsValue {
    JsValue::Number(first_number(&args).floor())
}

fn math_ceil(args: Vec<JsValue>) -> JsValue {
    JsValue::Number(first_number(&args).ceil())
}

fn math_round(args: Vec<JsValue>) -> JsValue {
    JsValue::Number(first_number(&args).round())
}

fn math_max(args: Vec<JsValue>) -> JsValue {
    JsValue::Number(
        args.iter()
            .map(JsValue::to_number)
            .fold(f64::NEG_INFINITY, f64::max),
    )
}

fn math_min(args: Vec<JsValue>) -> JsValue {
    JsValue::Number(
        args.iter()
            .map(JsValue::to_number)
            .fold(f64::INFINITY, f64::min),
    )
}

fn math_pow(args: Vec<JsValue>) -> JsValue {
    let base = args.first().map(JsValue::to_number).unwrap_or(f64::NAN);
    let exponent = args.get(1).map(JsValue::to_number).unwrap_or(f64::NAN);
    JsValue::Number(base.powf(exponent))
}

fn object_keys(args: Vec<JsValue>) -> JsValue {
    match args.first() {
        Some(JsValue::Object(map)) => {
            JsValue::Array(map.keys().cloned().map(JsValue::String).collect())
        }
        Some(JsValue::Error(err)) => JsValue::Array({
            let mut keys = vec![
                JsValue::String("name".to_string()),
                JsValue::String("message".to_string()),
            ];
            keys.extend(err.properties.keys().cloned().map(JsValue::String));
            keys
        }),
        _ => JsValue::Array(vec![]),
    }
}

fn object_values(args: Vec<JsValue>) -> JsValue {
    match args.first() {
        Some(JsValue::Object(map)) => JsValue::Array(map.values().cloned().collect()),
        Some(JsValue::Error(err)) => JsValue::Array({
            let mut values = vec![
                JsValue::String(err.kind.as_str().to_string()),
                JsValue::String(err.message.clone()),
            ];
            values.extend(err.properties.values().cloned());
            values
        }),
        _ => JsValue::Array(vec![]),
    }
}

fn object_assign(args: Vec<JsValue>) -> JsValue {
    let mut target = match args.first() {
        Some(JsValue::Object(map)) => map.clone(),
        _ => HashMap::new(),
    };
    for source in args.iter().skip(1) {
        if let JsValue::Object(map) = source {
            for (key, value) in map {
                target.insert(key.clone(), value.clone());
            }
        }
    }
    JsValue::Object(target)
}

fn json_stringify(args: Vec<JsValue>) -> JsValue {
    JsValue::String(
        args.first()
            .map(stringify_json)
            .unwrap_or_else(|| "undefined".to_string()),
    )
}

fn stringify_json(value: &JsValue) -> String {
    match value {
        JsValue::Number(n) => n.to_string(),
        JsValue::String(s) => format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"")),
        JsValue::Boolean(b) => b.to_string(),
        JsValue::Null => "null".to_string(),
        JsValue::Undefined | JsValue::Function(_) => "undefined".to_string(),
        JsValue::Array(values) => {
            let values = values
                .iter()
                .map(stringify_json)
                .collect::<Vec<_>>()
                .join(",");
            format!("[{}]", values)
        }
        JsValue::Object(map) => {
            let entries = map
                .iter()
                .map(|(key, value)| {
                    format!("\"{}\":{}", key.replace('"', "\\\""), stringify_json(value))
                })
                .collect::<Vec<_>>()
                .join(",");
            format!("{{{}}}", entries)
        }
        JsValue::Error(err) => stringify_json(&JsValue::Object(HashMap::from([
            (
                "name".to_string(),
                JsValue::String(err.kind.as_str().to_string()),
            ),
            ("message".to_string(), JsValue::String(err.message.clone())),
        ]))),
    }
}

fn json_parse(args: Vec<JsValue>) -> JsValue {
    let Some(JsValue::String(input)) = args.first() else {
        return JsValue::Undefined;
    };
    parse_json_value(input.trim()).unwrap_or(JsValue::Undefined)
}

fn parse_json_value(input: &str) -> Option<JsValue> {
    if input == "null" {
        Some(JsValue::Null)
    } else if input == "true" {
        Some(JsValue::Boolean(true))
    } else if input == "false" {
        Some(JsValue::Boolean(false))
    } else if input.starts_with('"') && input.ends_with('"') {
        Some(JsValue::String(
            input[1..input.len() - 1]
                .replace("\\\"", "\"")
                .replace("\\\\", "\\"),
        ))
    } else {
        input.parse::<f64>().ok().map(JsValue::Number)
    }
}

pub(crate) fn call_method(
    receiver: &JsValue,
    method_name: &str,
    args: Vec<JsValue>,
    scope: Rc<RefCell<JsScope>>,
) -> (JsValue, JsValue) {
    match receiver {
        JsValue::Array(arr) => {
            let mut new_array = arr.clone();
            match method_name {
                "push" => {
                    if let Some(value) = args.first() {
                        new_array.push(value.clone());
                        (
                            JsValue::Number(new_array.len() as f64),
                            JsValue::Array(new_array),
                        )
                    } else {
                        (JsValue::Undefined, JsValue::Array(new_array))
                    }
                }
                "pop" => {
                    let popped = new_array.pop().unwrap_or(JsValue::Undefined);
                    (popped, JsValue::Array(new_array))
                }
                "map" => {
                    if let Some(JsValue::Function(callback)) = args.first() {
                        let mapped: Vec<JsValue> = arr
                            .iter()
                            .enumerate()
                            .map(|(index, value)| {
                                let call_args = vec![
                                    value.clone(),
                                    JsValue::Number(index as f64),
                                    JsValue::Array(arr.clone()),
                                ];
                                callback.execute(&call_args, Rc::clone(&scope), None)
                            })
                            .collect();
                        (JsValue::Array(mapped), receiver.clone())
                    } else {
                        (JsValue::Undefined, receiver.clone())
                    }
                }
                "filter" => {
                    if let Some(JsValue::Function(callback)) = args.first() {
                        let filtered = arr
                            .iter()
                            .enumerate()
                            .filter_map(|(index, value)| {
                                let call_args = vec![
                                    value.clone(),
                                    JsValue::Number(index as f64),
                                    JsValue::Array(arr.clone()),
                                ];
                                callback
                                    .execute(&call_args, Rc::clone(&scope), None)
                                    .to_bool()
                                    .then(|| value.clone())
                            })
                            .collect();
                        (JsValue::Array(filtered), receiver.clone())
                    } else {
                        (JsValue::Undefined, receiver.clone())
                    }
                }
                "forEach" => {
                    if let Some(JsValue::Function(callback)) = args.first() {
                        for (index, value) in arr.iter().enumerate() {
                            let call_args = vec![
                                value.clone(),
                                JsValue::Number(index as f64),
                                JsValue::Array(arr.clone()),
                            ];
                            callback.execute(&call_args, Rc::clone(&scope), None);
                        }
                    }
                    (JsValue::Undefined, receiver.clone())
                }
                "reduce" => {
                    if let Some(JsValue::Function(callback)) = args.first() {
                        let mut iter = arr.iter().cloned().enumerate();
                        let mut accumulator = if let Some(initial) = args.get(1) {
                            initial.clone()
                        } else if let Some((_, first)) = iter.next() {
                            first
                        } else {
                            return (JsValue::Undefined, receiver.clone());
                        };
                        for (index, value) in iter {
                            let call_args = vec![
                                accumulator,
                                value.clone(),
                                JsValue::Number(index as f64),
                                JsValue::Array(arr.clone()),
                            ];
                            accumulator = callback.execute(&call_args, Rc::clone(&scope), None);
                        }
                        (accumulator, receiver.clone())
                    } else {
                        (JsValue::Undefined, receiver.clone())
                    }
                }
                "includes" => {
                    let needle = args.first().cloned().unwrap_or(JsValue::Undefined);
                    let found = arr.iter().any(|value| {
                        matches!(value.strict_equals(&needle), JsValue::Boolean(true))
                    });
                    (JsValue::Boolean(found), receiver.clone())
                }
                "join" => {
                    let separator = args
                        .first()
                        .map(JsValue::to_primitive_string)
                        .unwrap_or_else(|| ",".to_string());
                    let joined = arr
                        .iter()
                        .map(JsValue::to_primitive_string)
                        .collect::<Vec<_>>()
                        .join(&separator);
                    (JsValue::String(joined), receiver.clone())
                }
                _ => (JsValue::Undefined, receiver.clone()),
            }
        }
        JsValue::Object(_) => {
            let method_property = receiver.get_property(method_name);
            if let JsValue::Function(func) = method_property {
                let this_context = Rc::new(RefCell::new(receiver.clone()));
                let result = func.execute(&args, scope, Some(Rc::clone(&this_context)));
                (result, this_context.borrow().clone())
            } else {
                (JsValue::Undefined, receiver.clone())
            }
        }
        _ => (JsValue::Undefined, receiver.clone()),
    }
}
