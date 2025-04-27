use wasm_bindgen::{prelude::wasm_bindgen, JsValue};
use web_sys::js_sys::Reflect;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(thread_local_v2, js_name = "env", js_namespace = process)]
    static ENV: JsValue;
}

pub struct Env(JsValue);

fn env() -> Env {
    Env::new()
}

pub fn var(key: &str) -> Option<String> {
    env().get(key)
}

impl Env {
    fn new() -> Self {
        Self(ENV.with(|env| env.clone()))
    }

    fn get(&self, key: &str) -> Option<String> {
        if let Ok(value) = Reflect::get(&self.0, &JsValue::from_str(key)) {
            if value.is_string() {
                value.as_string()
            } else {
                None
            }
        } else {
            None
        }
    }
}
