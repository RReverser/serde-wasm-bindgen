use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;

mod color;
mod prim_str;

mod canada;
mod citm_catalog;
mod twitter;

use canada::Canada;
use citm_catalog::CitmCatalog;
use twitter::Twitter;

#[wasm_bindgen(start)]
pub fn init_console() {
    console_error_panic_hook::set_once();
}

#[cfg(feature = "serde-wasm-bindgen")]
#[wasm_bindgen]
pub fn parse_canada_with_serde_wasm_bindgen(input: JsValue) -> *const Canada {
    Box::leak(serde_wasm_bindgen::from_value(input).unwrap())
}

#[cfg(feature = "serde-json")]
#[wasm_bindgen]
pub fn parse_canada_with_serde_json(input: JsValue) -> *const Canada {
    Box::leak(input.into_serde().unwrap())
}

#[cfg(feature = "serde-wasm-bindgen")]
#[wasm_bindgen]
pub fn serialize_canada_with_serde_wasm_bindgen(input: *const Canada) -> JsValue {
    serde_wasm_bindgen::to_value(unsafe { &*input }).unwrap()
}

#[cfg(feature = "serde-json")]
#[wasm_bindgen]
pub fn serialize_canada_with_serde_json(input: *const Canada) -> JsValue {
    JsValue::from_serde(unsafe { &*input }).unwrap()
}

#[wasm_bindgen]
pub fn free_canada(input: *mut Canada) {
    unsafe { Box::from_raw(input); }
}

#[cfg(feature = "serde-wasm-bindgen")]
#[wasm_bindgen]
pub fn parse_citm_catalog_with_serde_wasm_bindgen(input: JsValue) -> *const CitmCatalog {
    Box::leak(serde_wasm_bindgen::from_value(input).unwrap())
}

#[cfg(feature = "serde-json")]
#[wasm_bindgen]
pub fn parse_citm_catalog_with_serde_json(input: JsValue) -> *const CitmCatalog {
    Box::leak(input.into_serde().unwrap())
}

#[cfg(feature = "serde-wasm-bindgen")]
#[wasm_bindgen]
pub fn serialize_citm_catalog_with_serde_wasm_bindgen(input: *const CitmCatalog) -> JsValue {
    serde_wasm_bindgen::to_value(unsafe { &*input }).unwrap()
}

#[cfg(feature = "serde-json")]
#[wasm_bindgen]
pub fn serialize_citm_catalog_with_serde_json(input: *const CitmCatalog) -> JsValue {
    JsValue::from_serde(unsafe { &*input }).unwrap()
}

#[wasm_bindgen]
pub fn free_citm_catalog(input: *mut CitmCatalog) {
    unsafe { Box::from_raw(input); }
}

#[cfg(feature = "serde-wasm-bindgen")]
#[wasm_bindgen]
pub fn parse_twitter_with_serde_wasm_bindgen(input: JsValue) -> *const Twitter {
    Box::leak(serde_wasm_bindgen::from_value(input).unwrap())
}

#[cfg(feature = "serde-json")]
#[wasm_bindgen]
pub fn parse_twitter_with_serde_json(input: JsValue) -> *const Twitter {
    Box::leak(input.into_serde().unwrap())
}

#[cfg(feature = "serde-wasm-bindgen")]
#[wasm_bindgen]
pub fn serialize_twitter_with_serde_wasm_bindgen(input: *const Twitter) -> JsValue {
    serde_wasm_bindgen::to_value(unsafe { &*input }).unwrap()
}

#[cfg(feature = "serde-json")]
#[wasm_bindgen]
pub fn serialize_twitter_with_serde_json(input: *const Twitter) -> JsValue {
    JsValue::from_serde(unsafe { &*input }).unwrap()
}

#[wasm_bindgen]
pub fn free_twitter(input: *mut Twitter) {
    unsafe { Box::from_raw(input); }
}
