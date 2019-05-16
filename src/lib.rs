#![cfg_attr(feature = "external_doc", feature(external_doc))]
#![cfg_attr(feature = "external_doc", doc(include = "../README.md"))]
#![cfg_attr(feature = "external_doc", warn(missing_docs))]

use wasm_bindgen::prelude::*;

mod de;
mod error;
mod ser;

pub use de::Deserializer;
pub use error::Error;
pub use ser::Serializer;

type Result<T> = std::result::Result<T, Error>;

fn static_str_to_js(s: &'static str) -> JsValue {
    thread_local! {
        static CACHE: std::cell::RefCell<fnv::FnvHashMap<&'static str, JsValue>> = Default::default();
    }
    CACHE.with(|cache| {
        cache
            .borrow_mut()
            .entry(s)
            .or_insert_with(|| JsValue::from_str(s))
            .clone()
    })
}

/// Converts [`JsValue`] into a Rust type.
pub fn from_value<T: serde::de::DeserializeOwned>(value: JsValue) -> Result<T> {
    T::deserialize(Deserializer::from(value))
}

/// Converts a Rust value into a [`JsValue`].
pub fn to_value<T: serde::ser::Serialize>(value: &T) -> Result<JsValue> {
    value.serialize(&Serializer::new())
}
