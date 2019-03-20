extern crate js_sys;
extern crate serde;
extern crate wasm_bindgen;

use wasm_bindgen::{JsCast, JsValue};

pub mod de;
pub mod ser;

pub use de::Deserializer;
pub use ser::Serializer;

pub use serde::de::value::Error;

/// Stringifies a JS error into a [`serde::de::Error::custom`].
#[cold]
fn convert_error(err: JsValue) -> Error {
    serde::de::Error::custom(String::from(
        err.unchecked_into::<js_sys::Object>().to_string(),
    ))
}

pub fn from_value<T: serde::de::DeserializeOwned>(value: JsValue) -> Result<T, Error> {
    T::deserialize(Deserializer::from(value))
}
