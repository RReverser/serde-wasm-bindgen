use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;

mod color;
mod prim_str;

mod canada;
mod citm_catalog;
mod twitter;

#[wasm_bindgen]
#[derive(Serialize, Deserialize)]
pub struct Canada(canada::Canada);

#[wasm_bindgen]
#[derive(Serialize, Deserialize)]
pub struct CitmCatalog(citm_catalog::CitmCatalog);

#[wasm_bindgen]
#[derive(Serialize, Deserialize)]
pub struct Twitter(twitter::Twitter);

#[wasm_bindgen(start)]
pub fn init_console() {
    console_error_panic_hook::set_once();
}

// Like serde_wasm_bindgen_to_value but with JSON-like output (no Maps).
fn serde_wasm_bindgen_to_value(
    value: &impl Serialize,
) -> Result<JsValue, serde_wasm_bindgen::Error> {
    let serializer = serde_wasm_bindgen::Serializer::new().serialize_maps_as_objects(true);
    value.serialize(&serializer)
}

#[cfg(feature = "serde-wasm-bindgen")]
#[wasm_bindgen]
pub fn parse_canada_with_serde_wasm_bindgen(input: JsValue) -> Canada {
    serde_wasm_bindgen::from_value(input).unwrap()
}

#[cfg(feature = "serde-json")]
#[wasm_bindgen]
#[allow(deprecated)]
pub fn parse_canada_with_serde_json(input: JsValue) -> Canada {
    input.into_serde().unwrap()
}

#[cfg(feature = "serde-wasm-bindgen")]
#[wasm_bindgen]
pub fn serialize_canada_with_serde_wasm_bindgen(input: &Canada) -> JsValue {
    serde_wasm_bindgen_to_value(input).unwrap()
}

#[cfg(feature = "serde-json")]
#[wasm_bindgen]
#[allow(deprecated)]
pub fn serialize_canada_with_serde_json(input: &Canada) -> JsValue {
    JsValue::from_serde(input).unwrap()
}

#[cfg(feature = "serde-wasm-bindgen")]
#[wasm_bindgen]
pub fn parse_citm_catalog_with_serde_wasm_bindgen(input: JsValue) -> CitmCatalog {
    serde_wasm_bindgen::from_value(input).unwrap()
}

#[cfg(feature = "serde-json")]
#[wasm_bindgen]
#[allow(deprecated)]
pub fn parse_citm_catalog_with_serde_json(input: JsValue) -> CitmCatalog {
    input.into_serde().unwrap()
}

#[cfg(feature = "serde-wasm-bindgen")]
#[wasm_bindgen]
pub fn serialize_citm_catalog_with_serde_wasm_bindgen(input: &CitmCatalog) -> JsValue {
    serde_wasm_bindgen_to_value(input).unwrap()
}

#[cfg(feature = "serde-json")]
#[wasm_bindgen]
#[allow(deprecated)]
pub fn serialize_citm_catalog_with_serde_json(input: &CitmCatalog) -> JsValue {
    JsValue::from_serde(input).unwrap()
}

#[cfg(feature = "serde-wasm-bindgen")]
#[wasm_bindgen]
pub fn parse_twitter_with_serde_wasm_bindgen(input: JsValue) -> Twitter {
    serde_wasm_bindgen::from_value(input).unwrap()
}

#[cfg(feature = "serde-json")]
#[wasm_bindgen]
#[allow(deprecated)]
pub fn parse_twitter_with_serde_json(input: JsValue) -> Twitter {
    input.into_serde().unwrap()
}

#[cfg(feature = "serde-wasm-bindgen")]
#[wasm_bindgen]
pub fn serialize_twitter_with_serde_wasm_bindgen(input: &Twitter) -> JsValue {
    serde_wasm_bindgen_to_value(input).unwrap()
}

#[cfg(feature = "serde-json")]
#[wasm_bindgen]
#[allow(deprecated)]
pub fn serialize_twitter_with_serde_json(input: &Twitter) -> JsValue {
    JsValue::from_serde(input).unwrap()
}
