#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
#![warn(clippy::missing_const_for_fn)]

use std::convert::TryFrom;

use js_sys::JsString;
use wasm_bindgen::{
    convert::{FromWasmAbi, IntoWasmAbi},
    prelude::*,
    JsCast,
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

/// A wrapper around a [`JsValue`] (or anything that can be cast into one)
/// that makes it pass through serialization and deserialization unchanged.
///
/// # Example
/// ```rust
/// use serde_wasm_bindgen::{PreservedValue, to_value};
/// #[derive(serde::Serialize)]
/// struct MyStruct {
///     int_field: i32,
///     js_field: PreservedValue<JsValue>,
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
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(
    into = "PreservedValueWrapper",
    try_from = "PreservedValueWrapper",
    bound = "T : JsCast + Clone"
)]
pub struct PreservedValue<T: JsCast + Clone>(pub T);

// Some arbitrary string that no one will collide with unless they try.
pub(crate) const PRESERVED_VALUE_MAGIC: &str = "1fc430ca-5b7f-4295-92de-33cf2b145d38";

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename = "1fc430ca-5b7f-4295-92de-33cf2b145d38")]
struct PreservedValueWrapper(u32);

impl<T: JsCast + Clone> From<PreservedValue<T>> for PreservedValueWrapper {
    fn from(val: PreservedValue<T>) -> Self {
        PreservedValueWrapper(val.0.into().into_abi())
    }
}

impl<T: JsCast + Clone> TryFrom<PreservedValueWrapper> for PreservedValue<T> {
    // JsValue would be more natural, but serde insists on a `Display` error.
    type Error = String;

    fn try_from(wrap: PreservedValueWrapper) -> std::result::Result<Self, String> {
        // When used with our deserializer this unsafe is correct, because the
        // deserializer just converted a JsValue into_abi.
        // With other deserializers, this may be incorrect but it shouldn't be UB
        // because JsValues are represented using indices into a JS-side (i.e.
        // bounds-checked) array.
        let val: JsValue = unsafe { FromWasmAbi::from_abi(wrap.0) };
        val.dyn_into()
            .map(|val| PreservedValue(val))
            .map_err(|e| format!("incompatible JS value {e:?}"))
    }
}
