#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

use js_sys::JsString;
use wasm_bindgen::prelude::*;

mod bindings;
mod de;
mod error;
mod ser;

pub use de::Deserializer;
pub use error::Error;
pub use ser::Serializer;

type Result<T> = std::result::Result<T, Error>;

fn static_str_to_js(s: &'static str) -> JsString {
    thread_local! {
        // Since we're mainly optimising for converting the exact same string literal over and over again,
        // which will always have the same pointer, we can speed things up by indexing by the string's pointer
        // instead of its value.
        static CACHE: std::cell::RefCell<fnv::FnvHashMap<*const str, JsString>> = Default::default();
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
pub fn to_value<T: serde::ser::Serialize + ?Sized>(value: &T) -> Result<JsValue> {
    value.serialize(&Serializer::new())
}
