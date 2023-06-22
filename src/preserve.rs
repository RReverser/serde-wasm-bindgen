use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};
use std::{fmt, sync::Mutex};
use wasm_bindgen::JsValue;

#[derive(Debug, Clone)]
pub struct PreserveJsValue(pub JsValue);
unsafe impl Send for PreserveJsValue {}
unsafe impl Sync for PreserveJsValue {}

pub(crate) static NEXT_PRESERVE: Mutex<Option<PreserveJsValue>> = Mutex::new(None);

impl Serialize for PreserveJsValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        NEXT_PRESERVE.lock().unwrap().replace(self.clone());
        serializer.serialize_i64(0)
    }
}

impl<'de> Deserialize<'de> for PreserveJsValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct JsValueVisitor;

        impl<'de> Visitor<'de> for JsValueVisitor {
            type Value = PreserveJsValue;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct PreserveJsValue")
            }

            fn visit_i64<E>(self, _value: i64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(NEXT_PRESERVE.lock().unwrap().take().unwrap())
            }
        }

        deserializer.deserialize_i64(JsValueVisitor)
    }
}
