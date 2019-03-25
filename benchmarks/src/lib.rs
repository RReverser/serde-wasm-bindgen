use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;

mod color;
mod prim_str;

mod canada;
mod citm_catalog;
mod twitter;

#[wasm_bindgen(start)]
pub fn init_console() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn parse_canada_with_serde_wasm_bindgen(input: JsValue) {
    let _: canada::Canada = serde_wasm_bindgen::from_value(input).unwrap();
}

#[wasm_bindgen]
pub fn parse_canada_with_serde_json(input: JsValue) {
    let _: canada::Canada = input.into_serde().unwrap();
}

#[wasm_bindgen]
pub fn parse_citm_catalog_with_serde_wasm_bindgen(input: JsValue) {
    let _: citm_catalog::CitmCatalog = serde_wasm_bindgen::from_value(input).unwrap();
}

#[wasm_bindgen]
pub fn parse_citm_catalog_with_serde_json(input: JsValue) {
    let _: citm_catalog::CitmCatalog = input.into_serde().unwrap();
}

#[wasm_bindgen]
pub fn parse_twitter_with_serde_wasm_bindgen(input: JsValue) {
    let _: twitter::Twitter = serde_wasm_bindgen::from_value(input).unwrap();
}

#[wasm_bindgen]
pub fn parse_twitter_with_serde_json(input: JsValue) {
    let _: twitter::Twitter = input.into_serde().unwrap();
}
