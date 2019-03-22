use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;

use std::collections::BTreeMap as Map;

pub type Canada = FeatureCollection;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FeatureCollection {
    #[serde(rename = "type")]
    pub obj_type: ObjType,
    pub features: Vec<Feature>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Feature {
    #[serde(rename = "type")]
    pub obj_type: ObjType,
    pub properties: Map<String, String>,
    pub geometry: Geometry,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Geometry {
    #[serde(rename = "type")]
    pub obj_type: ObjType,
    pub coordinates: Vec<Vec<(Latitude, Longitude)>>,
}

pub type Latitude = f32;
pub type Longitude = f32;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ObjType {
	FeatureCollection,
	Feature,
	Polygon,
}

#[wasm_bindgen]
pub fn init_console() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn parse_canada_with_serde_wasm_bindgen(input: JsValue) {
    let _: Canada = serde_wasm_bindgen::from_value(input).unwrap();
}

#[wasm_bindgen]
pub fn parse_canada_with_serde_json(input: JsValue) {
    let _: Canada = input.into_serde().unwrap();
}
