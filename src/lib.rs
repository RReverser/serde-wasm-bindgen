use wasm_bindgen::prelude::*;

pub mod de;
pub mod error;
pub mod ser;

pub use de::Deserializer;
pub use error::Error;
pub use ser::Serializer;

pub type Result<T> = std::result::Result<T, Error>;

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

pub fn from_value<T: serde::de::DeserializeOwned>(value: JsValue) -> Result<T> {
    T::deserialize(Deserializer::from(value))
}

pub fn to_value<T: serde::ser::Serialize>(value: &T) -> Result<JsValue> {
    value.serialize(&Serializer::new())
}
