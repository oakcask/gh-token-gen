use wasm_bindgen::{prelude::wasm_bindgen, JsValue};
use web_sys::js_sys::Reflect;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(thread_local_v2, js_name = "env", js_namespace = process)]
    static ENV: JsValue;
}

pub fn var(key: &str) -> Option<String> {
    ENV.with(|env| {
        if let Ok(value) = Reflect::get(env, &JsValue::from_str(key)) {
            if value.is_string() {
                value.as_string()
            } else {
                None
            }
        } else {
            None
        }
    })
}

#[cfg(test)]
fn set_var(key: &str, value: &str) {
    ENV.with(|env| {
        Reflect::set(env, &JsValue::from_str(key), &JsValue::from_str(value)).unwrap();
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[wasm_bindgen_test::wasm_bindgen_test]
    fn test_var() {
        set_var("TEST_KEY", "TEST_VALUE");
        assert_eq!(var("TEST_KEY"), Some("TEST_VALUE".to_string()));
    }
}
