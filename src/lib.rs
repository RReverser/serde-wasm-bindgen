//! An alternative to the built-in support within `wasm-bindgen` for converting
//! Rust types into `JsValue`s. This alternative avoids the intermediate
//! stringification of Rust values and thus is more efficient.
//!
//! # Usage
//!
//! To serialize a Rust value into a `JsValue`:
//!
//! ```
//! use wasm_bindgen::JsValue;
//! use serde::Serialize;
//! use serde_wasm_bindgen as swb;
//!
//! #[derive(Serialize)]
//! struct Foo {
//!   num: usize,
//! }
//!
//! pub fn pass_value_to_js() -> Result<JsValue, swb::Error> {
//!   let foo = Foo { num: 37 };
//!   swb::to_value(&foo)
//! }
//! ```
//!
//! Likewise, the [`from_value`] function can be used for deserialization.

#![warn(missing_docs)]

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
pub fn to_value<T: serde::ser::Serialize + ?Sized>(value: &T) -> Result<JsValue> {
    value.serialize(&Serializer::new())
}
