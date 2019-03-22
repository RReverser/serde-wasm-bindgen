use js_sys::{ArrayBuffer, JsString, Number, Object, Uint8Array};
use serde::{de, serde_if_integer128};
use wasm_bindgen::{JsCast, JsValue};

use super::{convert_error, Error, Result};

/// Provides [`de::SeqAccess`] from any JS iterator.
struct SeqAccess {
    iter: js_sys::IntoIter,
}

impl<'de> de::SeqAccess<'de> for SeqAccess {
    type Error = Error;

    fn next_element_seed<T: de::DeserializeSeed<'de>>(
        &mut self,
        seed: T,
    ) -> Result<Option<T::Value>> {
        match self.iter.next() {
            Some(Ok(value)) => Ok(Some(seed.deserialize(Deserializer::from(value))?)),
            Some(Err(err)) => Err(convert_error(err)),
            None => Ok(None),
        }
    }
}

/// Provides [`serde::de::MapAccess`] from any JS iterator that returns `[key, value]` pairs.
struct MapAccess {
    iter: js_sys::IntoIter,
    next_value: Option<Deserializer>,
}

impl<'de> de::MapAccess<'de> for MapAccess {
    type Error = Error;

    fn next_key_seed<K: de::DeserializeSeed<'de>>(&mut self, seed: K) -> Result<Option<K::Value>> {
        match self.iter.next() {
            Some(Ok(pair)) => {
                debug_assert!(self.next_value.is_none());
                let (key, value) = convert_pair(pair)?;
                self.next_value = Some(value);
                Ok(Some(seed.deserialize(key)?))
            }
            Some(Err(err)) => Err(convert_error(err)),
            None => Ok(None),
        }
    }

    fn next_value_seed<V: de::DeserializeSeed<'de>>(&mut self, seed: V) -> Result<V::Value> {
        seed.deserialize(self.next_value.take().unwrap())
    }
}

/// Provides [`serde::de::EnumAccess`] from given JS values for the `tag` and the `payload`.
struct EnumAccess {
    tag: Deserializer,
    payload: Deserializer,
}

impl<'de> de::EnumAccess<'de> for EnumAccess {
    type Error = Error;
    type Variant = Deserializer;

    fn variant_seed<V: de::DeserializeSeed<'de>>(
        self,
        seed: V,
    ) -> Result<(V::Value, Self::Variant)> {
        Ok((seed.deserialize(self.tag)?, self.payload))
    }
}

/// A newtype that allows using any [`JsValue`] as a [`serde::Deserializer`].
pub struct Deserializer {
    value: JsValue,
}

impl From<JsValue> for Deserializer {
    fn from(value: JsValue) -> Self {
        Self { value }
    }
}

/// Destructures a JS `[key, value]` pair into a tuple of [`Deserializer`]s.
fn convert_pair(pair: JsValue) -> Result<(Deserializer, Deserializer)> {
    Ok((
        js_sys::Reflect::get_u32(&pair, 0)
            .map(Deserializer::from)
            .map_err(convert_error)?,
        js_sys::Reflect::get_u32(&pair, 1)
            .map(Deserializer::from)
            .map_err(convert_error)?,
    ))
}

impl Deserializer {
    /// Casts the internal value into an object, including support for prototype-less objects.
    /// See https://github.com/rustwasm/wasm-bindgen/issues/1366 for why we don't use `dyn_ref`.
    fn as_object(&self) -> Option<&Object> {
        if self.value.is_object() {
            Some(self.value.unchecked_ref())
        } else {
            None
        }
    }

    fn is_nullish(&self) -> bool {
        self.value.is_null() || self.value.is_undefined()
    }

    fn as_bytes(&self) -> Option<Vec<u8>> {
        let temp;

        let v = if let Some(v) = self.value.dyn_ref::<Uint8Array>() {
            v
        } else if let Some(v) = self.value.dyn_ref::<ArrayBuffer>() {
            temp = Uint8Array::new(v);
            &temp
        } else {
            return None;
        };

        let mut vec = Vec::with_capacity(v.byte_length() as _);
        unsafe {
            vec.set_len(v.byte_length() as _);
            v.copy_to(vec.as_mut_slice());
        }
        Some(vec)
    }

    /// Converts any JS string into a [`JsString`], while avoiding `instanceof String`.
    /// See https://github.com/rustwasm/wasm-bindgen/issues/1367 for why we don't use `dyn_ref`.
    fn as_js_string(&self) -> Option<&JsString> {
        if self.value.is_string() {
            Some(self.value.unchecked_ref())
        } else {
            None
        }
    }

    #[cold]
    fn invalid_type<'de, V: de::Visitor<'de>>(&self, visitor: V) -> Result<V::Value> {
        let string;
        let bytes;

        let unexpected = if self.is_nullish() {
            de::Unexpected::Unit
        } else if let Some(v) = self.value.as_bool() {
            de::Unexpected::Bool(v)
        } else if let Some(v) = self.value.as_f64() {
            de::Unexpected::Float(v)
        } else if let Some(v) = self.value.as_string() {
            string = v;
            de::Unexpected::Str(&string)
        } else if let Some(v) = self.as_bytes() {
            bytes = v;
            de::Unexpected::Bytes(&bytes)
        } else {
            string = format!("{:?}", self.value);
            de::Unexpected::Other(&string)
        };

        Err(de::Error::invalid_type(unexpected, &visitor))
    }

    fn as_safe_integer(&self) -> Option<i64> {
        if let Some(v) = self.value.as_f64() {
            if Number::is_safe_integer(&self.value) {
                return Some(v as i64);
            }
        }
        None
    }
}

impl<'de> de::Deserializer<'de> for Deserializer {
    type Error = Error;

    fn deserialize_any<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        if self.is_nullish() {
            // Ideally we would only treat `undefined` as `()` / `None` which would be semantically closer
            // to JS definitions, but, unfortunately, WebIDL generates missing values as `null`
            // and we probably want to support these as well.
            visitor.visit_unit()
        } else if let Some(v) = self.value.as_bool() {
            visitor.visit_bool(v)
        } else if let Some(v) = self.value.as_f64() {
            visitor.visit_f64(v)
        } else if let Some(v) = self.value.as_string() {
            visitor.visit_string(v)
        } else {
            self.invalid_type(visitor)
        }
    }

    fn deserialize_unit<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        if self.is_nullish() {
            visitor.visit_unit()
        } else {
            self.invalid_type(visitor)
        }
    }

    fn deserialize_unit_struct<V: de::Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value> {
        self.deserialize_unit(visitor)
    }

    fn deserialize_bool<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        if let Some(v) = self.value.as_bool() {
            visitor.visit_bool(v)
        } else {
            self.invalid_type(visitor)
        }
    }

    // Serde happily converts `f64` to `f32` (with checks), so we can forward.
    fn deserialize_f32<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_f64(visitor)
    }

    fn deserialize_f64<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        if let Some(v) = self.value.as_f64() {
            visitor.visit_f64(v)
        } else {
            self.invalid_type(visitor)
        }
    }

    fn deserialize_identifier<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_str(visitor)
    }

    fn deserialize_str<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_string(visitor)
    }

    fn deserialize_string<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        if let Some(v) = self.value.as_string() {
            visitor.visit_string(v)
        } else {
            self.invalid_type(visitor)
        }
    }

    // Serde happily converts any integer to any integer (with checks), so let's forward all of
    // these to 64-bit methods to save some space in the generated WASM.

    fn deserialize_i8<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_i64(visitor)
    }

    fn deserialize_i16<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_i64(visitor)
    }

    fn deserialize_i32<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_i64(visitor)
    }

    serde_if_integer128! {
        fn deserialize_i128<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
            self.deserialize_i64(visitor)
        }
    }

    // Same as above, but for `i64`.

    fn deserialize_u8<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_u64(visitor)
    }

    fn deserialize_u16<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_u64(visitor)
    }

    fn deserialize_u32<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_u64(visitor)
    }

    serde_if_integer128! {
        fn deserialize_u128<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
            self.deserialize_u64(visitor)
        }
    }

    // Define real `i64` / `u64` deserializers that try to cast from `f64`.

    fn deserialize_i64<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match self.as_safe_integer() {
            Some(v) => visitor.visit_i64(v),
            None => self.invalid_type(visitor),
        }
    }

    fn deserialize_u64<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match self.as_safe_integer() {
            Some(v) if v >= 0 => visitor.visit_u64(v as _),
            _ => self.invalid_type(visitor),
        }
    }

    /// Converts a JavaScript string to a Rust char.
    ///
    /// By default we don't perform detection of single chars because it's pretty complicated,
    /// but if we get a hint that they're expected, this methods allows to avoid heap allocations
    /// of an intermediate `String` by directly converting numeric codepoints instead.
    fn deserialize_char<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        if let Some(s) = self.as_js_string() {
            let maybe_char = match s.length() {
                1 => {
                    // Should be valid `u16` because we checked the length.
                    std::char::from_u32(s.char_code_at(0) as u32)
                }
                2 => {
                    // Should be a valid `u32` by now.
                    let cp = s.code_point_at(0).as_f64().unwrap() as u32;
                    // This is a char only if it consists of two surrogates.
                    if cp > 0xFFFF {
                        std::char::from_u32(cp)
                    } else {
                        None
                    }
                }
                _ => None,
            };
            if let Some(c) = maybe_char {
                return visitor.visit_char(c);
            }
        }
        self.invalid_type(visitor)
    }

    // Serde can deserialize `visit_unit` into `None`, but can't deserialize arbitrary value
    // as `Some`, so we need to provide own simple implementation.
    fn deserialize_option<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        if !self.is_nullish() {
            visitor.visit_some(self)
        } else {
            visitor.visit_none()
        }
    }

    /// Simply calls `visit_newtype_struct`.
    fn deserialize_newtype_struct<V: de::Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value> {
        visitor.visit_newtype_struct(self)
    }

    /// Supported inputs:
    ///  - JS iterable (an object with [`[Symbol.iterator]`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Symbol/iterator)).
    /// Supported outputs:
    ///  - Any Rust sequence from Serde point of view ([`Vec`], [`HashSet`](std::collections::HashSet), etc.)
    fn deserialize_seq<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let iter = if js_sys::Array::is_array(&self.value) {
            self.value
                .unchecked_into::<js_sys::Array>()
                .values()
                .into_iter()
        } else if let Some(iter) = js_sys::try_iter(&self.value).map_err(convert_error)? {
            iter
        } else {
            return self.invalid_type(visitor);
        };
        visitor.visit_seq(SeqAccess { iter })
    }

    /// Forwards to [`Self::deserialize_seq`](#method.deserialize_seq).
    fn deserialize_tuple<V: de::Visitor<'de>>(self, _len: usize, visitor: V) -> Result<V::Value> {
        self.deserialize_seq(visitor)
    }

    /// Forwards to [`Self::deserialize_tuple`](#method.deserialize_tuple).
    fn deserialize_tuple_struct<V: de::Visitor<'de>>(
        self,
        _name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value> {
        self.deserialize_tuple(len, visitor)
    }

    /// Supported inputs:
    ///  - A JS iterable that is expected to return `[key, value]` pairs.
    ///  - A JS object, which will be iterated using [`Object.entries`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/entries).
    /// Supported outputs:
    ///  - A Rust key-value map ([`HashMap`](std::collections::HashMap), [`BTreeMap`](std::collections::BTreeMap), etc.).
    ///  - A typed Rust structure with `#[derive(Deserialize)]`.
    fn deserialize_map<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let map = MapAccess {
            iter: match js_sys::try_iter(&self.value).map_err(convert_error)? {
                Some(iter) => iter,
                None => match self.as_object() {
                    Some(obj) => Object::entries(obj).values().into_iter(),
                    None => return self.invalid_type(visitor),
                },
            },
            next_value: None,
        };
        visitor.visit_map(map)
    }

    /// Supports same input/output types as [`Self::deserialize_map`](#method.deserialize_map).
    fn deserialize_struct<V: de::Visitor<'de>>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        self.deserialize_map(visitor)
    }

    /// Here we try to be compatible with `serde-json`, which means supporting:
    ///  - `"Variant"` - gets converted to a unit variant `MyEnum::Variant`
    ///  - `{ Variant: ...payload... }` - gets converted to a `MyEnum::Variant { ...payload... }`.
    fn deserialize_enum<V: de::Visitor<'de>>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        let access = if self.value.is_string() {
            EnumAccess {
                tag: self.value.into(),
                payload: JsValue::UNDEFINED.into(),
            }
        } else if let Some(v) = self.as_object() {
            let entries = Object::entries(&v);
            if entries.length() != 1 {
                return Err(de::Error::invalid_length(entries.length() as _, &"1"));
            }
            let entry = js_sys::Reflect::get_u32(&entries, 0).map_err(convert_error)?;
            let (tag, payload) = convert_pair(entry)?;
            EnumAccess { tag, payload }
        } else {
            return self.invalid_type(visitor);
        };
        visitor.visit_enum(access)
    }

    /// Ignores any value without calling to the JS side even to check its type.
    fn deserialize_ignored_any<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_unit()
    }

    /// We can't take references to JS memory, so forwards to an owned [`Self::deserialize_byte_buf`](#method.deserialize_byte_buf).
    fn deserialize_bytes<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_byte_buf(visitor)
    }

    /// Serde expects `visit_byte_buf` to be called only in response to an explicit `deserialize_bytes`,
    /// so we provide conversions here.
    ///
    /// Supported inputs:
    ///  - `ArrayBuffer` - converted to an `Uint8Array` view first.
    ///  - `Uint8Array` - copied to a newly created `Vec<u8>` on the Rust side.
    fn deserialize_byte_buf<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        if let Some(bytes) = self.as_bytes() {
            visitor.visit_byte_buf(bytes)
        } else {
            self.invalid_type(visitor)
        }
    }

    fn is_human_readable(&self) -> bool {
        true
    }
}

impl<'de> de::VariantAccess<'de> for Deserializer {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        de::Deserialize::deserialize(self)
    }

    fn newtype_variant_seed<T: de::DeserializeSeed<'de>>(self, seed: T) -> Result<T::Value> {
        seed.deserialize(self)
    }

    fn tuple_variant<V: de::Visitor<'de>>(self, len: usize, visitor: V) -> Result<V::Value> {
        de::Deserializer::deserialize_tuple(self, len, visitor)
    }

    fn struct_variant<V: de::Visitor<'de>>(
        self,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        de::Deserializer::deserialize_struct(self, "", fields, visitor)
    }
}
