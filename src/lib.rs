#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
#![warn(clippy::missing_const_for_fn)]

use js_sys::JsString;
use serde::de::Visitor;
use wasm_bindgen::{
    convert::{FromWasmAbi, IntoWasmAbi},
    prelude::*,
};

mod de;
mod error;
mod ser;

pub use de::Deserializer;
pub use error::Error;
pub use ser::Serializer;

type Result<T> = std::result::Result<T, Error>;

fn static_str_to_js(s: &'static str) -> JsString {
    use std::cell::RefCell;
    use std::collections::HashMap;

    #[derive(Default)]
    struct PtrHasher {
        addr: usize,
    }

    impl std::hash::Hasher for PtrHasher {
        fn write(&mut self, _bytes: &[u8]) {
            unreachable!();
        }

        fn write_usize(&mut self, addr_or_len: usize) {
            if self.addr == 0 {
                self.addr = addr_or_len;
            }
        }

        fn finish(&self) -> u64 {
            self.addr as _
        }
    }

    type PtrBuildHasher = std::hash::BuildHasherDefault<PtrHasher>;

    thread_local! {
        // Since we're mainly optimising for converting the exact same string literal over and over again,
        // which will always have the same pointer, we can speed things up by indexing by the string's pointer
        // instead of its value.
        static CACHE: RefCell<HashMap<*const str, JsString, PtrBuildHasher>> = Default::default();
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
    fn get_with_ref_key(this: &ObjectExt, key: &JsString) -> JsValue;

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

pub(crate) const PRESERVED_VALUE_MAGIC: &str = "__serde_wasm_bindgen_PreservedValue";

/// A wrapper around a [`JsValue`] that makes it pass through serialization and deserialization unchanged.
///
/// # Example
/// ```rust
/// use serde_wasm_bindgen::{PreservedValue, to_value};
/// #[derive(serde::Serialize)]
/// struct MyStruct {
///     int_field: i32,
///     js_field: PreservedValue,
/// }
/// let big_array = js_sys::Int8Array::new_with_length(1000);
/// let s = MyStruct {
///     int_field: 5,
///     js_field: PreservedValue(big_array.into()),
/// };
///
/// // Will return a JsValue representing an object with two fields (`int_field` and `js_field`).
/// // `js_field` will be an `Int8Array` pointing to the same underlying JavaScript object as `big_array`.
/// to_value(&s);
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct PreservedValue(pub JsValue);

impl<'de> serde::Deserialize<'de> for PreservedValue {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visit;

        impl<'v> Visitor<'v> for Visit {
            type Value = PreservedValue;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("an integer pointing to a JsValue on the wasm heap")
            }

            fn visit_u32<E>(self, v: u32) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                // This is only correct for *our* deserializer. Some other deserializer
                // could call us with some weird value and we'd cast it to a JsValue index.
                // This would be wildly incorrect, but it should be safe because wasm-bindgen
                // bounds-checks its JsValue indices.
                Ok(PreservedValue(unsafe { JsValue::from_abi(v) }))
            }
        }

        deserializer.deserialize_newtype_struct(PRESERVED_VALUE_MAGIC, Visit)
    }
}

impl serde::Serialize for PreservedValue {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_newtype_struct(PRESERVED_VALUE_MAGIC, &self.0.clone().into_abi())
    }
}
