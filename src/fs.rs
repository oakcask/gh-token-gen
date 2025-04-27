use serde::Serialize;
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};
use web_sys::js_sys::Uint8Array;

use crate::error::Error;

#[wasm_bindgen(module = "node:fs")]
extern "C" {
    #[wasm_bindgen(catch, js_name = writeFileSync)]
    fn fs_write_file_sync(
        file: JsValue,
        data: JsValue,
        options: JsValue,
    ) -> Result<JsValue, JsValue>;
}

#[derive(Serialize)]
pub struct FileWriteOptions {
    pub encoding: &'static str,
    pub mode: u32,
    pub flag: &'static str,
}

pub fn write_file_sync(file: &str, data: &[u8], options: &FileWriteOptions) -> Result<(), Error> {
    let file = JsValue::from_str(file);
    let data = Uint8Array::from(data);
    let data = JsValue::from(data);
    let options = serde_wasm_bindgen::to_value(&options).map_err(Error::new)?;
    fs_write_file_sync(file, data, options)?;
    Ok(())
}
