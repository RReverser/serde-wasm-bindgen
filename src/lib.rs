extern crate js_sys;
extern crate serde;
extern crate wasm_bindgen;

use wasm_bindgen::JsValue;

pub mod de;
pub mod ser;

pub use de::Deserializer;
pub use ser::Serializer;

pub fn from_value<T: serde::de::DeserializeOwned>(value: JsValue) -> Result<T, de::Error> {
    T::deserialize(Deserializer::from(value))
}
