#![cfg_attr(feature = "external_doc", feature(external_doc))]
#![cfg_attr(feature = "external_doc", doc(include = "../README.md"))]
#![cfg_attr(feature = "external_doc", warn(missing_docs))]

use js_sys::JsString;
use wasm_bindgen::prelude::*;

mod de;
mod error;
mod ser;

pub use de::Deserializer;
pub use error::Error;
pub use ser::Serializer;

type Result<T> = std::result::Result<T, Error>;

fn static_str_to_js(s: &'static str) -> JsString {
    thread_local! {
        static CACHE: std::cell::RefCell<fnv::FnvHashMap<&'static str, JsString>> = Default::default();
    }
    CACHE.with(|cache| {
        cache
            .borrow_mut()
            .entry(s)
            .or_insert_with(|| s.into())
            .clone()
    })
}

/// Custom bindings to avoid using fallible `Reflect` for plain objects.
#[wasm_bindgen]
extern "C" {
    type ObjectExt;

    #[wasm_bindgen(method, indexing_getter)]
    fn get(this: &ObjectExt, key: JsString) -> JsValue;

    #[wasm_bindgen(method, indexing_setter)]
    fn set(this: &ObjectExt, key: JsString, value: JsValue);
}

/// Converts [`JsValue`] into a Rust type.
pub fn from_value<T: serde::de::DeserializeOwned>(value: JsValue) -> Result<T> {
    T::deserialize(Deserializer::from(value))
}

/// Converts a Rust value into a [`JsValue`].
pub fn to_value<T: serde::ser::Serialize>(value: &T) -> Result<JsValue> {
    value.serialize(&Serializer::new())
}
