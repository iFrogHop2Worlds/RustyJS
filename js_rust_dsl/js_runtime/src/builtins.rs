use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::{JsErrorKind, JsFunction, JsScope, JsValue, console_log};

pub(crate) fn resolve_global(name: &str) -> Option<JsValue> {
    if name == "console" {
        let mut obj = HashMap::new();
        obj.insert(
            "log".to_string(),
            JsValue::Function(JsFunction::new(
                vec!["values".to_string()],
                vec![],
                "log".to_string(),
            )),
        );
        return Some(JsValue::Object(obj));
    }
    None
}

pub(crate) fn call_global_function(name: &str, args: Vec<JsValue>) -> Option<JsValue> {
    match name {
        "Error" => {
            let message = args
                .first()
                .map(|v| v.to_primitive_string())
                .unwrap_or_default();
            Some(JsValue::new_error(JsErrorKind::Error, message))
        }
        "TypeError" => {
            let message = args
                .first()
                .map(|v| v.to_primitive_string())
                .unwrap_or_default();
            Some(JsValue::new_type_error(message))
        }
        _ => None,
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

pub(crate) fn try_call_named_method(
    object_name: &str,
    method: &str,
    args: Vec<JsValue>,
) -> Option<JsValue> {
    if object_name == "console" && method == "log" {
        console_log(args);
        return Some(JsValue::Undefined);
    }
    None
}
