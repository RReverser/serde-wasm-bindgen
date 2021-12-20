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
pub fn to_value<T: serde::ser::Serialize + ?Sized>(value: &T) -> Result<JsValue> {
    let ser = Serializer::new();
    to_value_with(value, &ser)
}

/// Converts a Rust value into a [`JsValue`] via some custom [`Serializer`].
///
/// With this you can customize serialization behaviour, a la:
///
/// ```
/// use serde_wasm_bindgen::Serializer;
/// use std::collections::HashMap;
/// use wasm_bindgen::JsValue;
///
/// fn as_objects(hm: &HashMap<String, usize>) -> Result<JsValue, serde_wasm_bindgen::Error> {
///     let ser = Serializer::new().serialize_maps_as_objects(true);
///     serde_wasm_bindgen::to_value_with(hm, &ser)
/// }
/// ```
pub fn to_value_with<T: serde::ser::Serialize + ?Sized>(
    value: &T,
    ser: &Serializer,
) -> Result<JsValue> {
    value.serialize(ser)
}
