use pem::Pem;
use rsa::pkcs1::DecodeRsaPrivateKey;
use serde::Serialize;
use wasm_actions_core::error::Error;
use std::str::FromStr;
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    js_sys::{ArrayBuffer, Object, Uint8Array},
    Crypto, CryptoKey,
};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(thread_local_v2, js_name = crypto)]
    static CRYPTO: Crypto;
}

#[derive(Serialize)]
struct RsaHashedImportParams {
    name: String,
    hash: String,
}

fn load_pkey(data: &str) -> Result<Vec<u8>, Error> {
    let pem = Pem::from_str(data).map_err(Error::new)?;
    match pem.tag() {
        "PRIVATE KEY" => Ok(pem.into_contents()),
        "RSA PRIVATE KEY" => pkcs1_to_pkcs8(data),
        _ => Err(Error::from(format!("unupported key format: {}", pem.tag()))),
    }
}

fn pkcs1_to_pkcs8(data: &str) -> Result<Vec<u8>, Error> {
    use rsa::pkcs8::EncodePrivateKey;
    let pkcs1 = rsa::RsaPrivateKey::from_pkcs1_pem(data).map_err(Error::new)?;
    let pkcs8 = pkcs1
        .to_pkcs8_pem(pkcs8::LineEnding::LF)
        .map_err(Error::new)?;
    Ok(Pem::from_str(&pkcs8).map_err(Error::new)?.into_contents())
}

pub async fn sign_sha256(buf: &[u8], pkey: &str) -> Result<Vec<u8>, Error> {
    let crypto = CRYPTO.with(|o| o.clone());
    let crypto = crypto.subtle();
    let pkey = load_pkey(pkey)?;
    let pkey = Uint8Array::from(pkey.as_slice()).buffer();
    let params = RsaHashedImportParams {
        name: String::from("RSASSA-PKCS1-v1_5"),
        hash: String::from("SHA-256"),
    };
    let params = serde_wasm_bindgen::to_value(&params).map_err(Error::new)?;
    let params = Object::from(params);
    let usages = vec![String::from("sign")];
    let usages = JsValue::from(usages);
    let pkey = crypto
        .import_key_with_object("pkcs8", pkey.as_ref(), &params, false, &usages)
        .map_err(Error::from)?;
    let pkey = JsFuture::from(pkey);
    let pkey = pkey.await.map_err(Error::from)?;
    let pkey: CryptoKey = pkey.into();

    let sign = crypto
        .sign_with_str_and_u8_array("RSASSA-PKCS1-v1_5", &pkey, buf)
        .map_err(Error::from)?;
    let sign = JsFuture::from(sign);
    let sign: ArrayBuffer = sign.await.map_err(Error::from)?.into();
    let sign = Uint8Array::new_with_byte_offset_and_length(&sign, 0, sign.byte_length());

    Ok(sign.to_vec())
}
