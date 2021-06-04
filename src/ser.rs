use js_sys::{Array, JsString, Map, Uint8Array};
use serde::ser::{self, Error as _, Serialize};
use wasm_bindgen::prelude::*;

use super::{static_str_to_js, Error};

type Result<T = JsValue> = super::Result<T>;

/// Custom bindings to avoid using fallible `Reflect` for plain objects.
#[wasm_bindgen]
extern "C" {
    type Object;

    #[wasm_bindgen(constructor)]
    fn new() -> Object;

    #[wasm_bindgen(method, indexing_setter)]
    fn set(this: &Object, key: JsValue, value: JsValue);
}

/// Wraps other serializers into an enum tagged variant form.
/// Uses {"Variant": ...payload...} for compatibility with serde-json.
pub struct VariantSerializer<S> {
    variant: &'static str,
    inner: S,
}

impl<S> VariantSerializer<S> {
    pub fn new(variant: &'static str, inner: S) -> Self {
        Self { variant, inner }
    }

    fn end(self, inner: impl FnOnce(S) -> Result) -> Result {
        let value = inner(self.inner)?;
        let obj = Object::new();
        obj.set(static_str_to_js(self.variant), value);
        Ok(obj.into())
    }
}

impl<S: ser::SerializeTupleStruct<Ok = JsValue, Error = Error>> ser::SerializeTupleVariant
    for VariantSerializer<S>
{
    type Ok = JsValue;
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
        self.inner.serialize_field(value)
    }

    fn end(self) -> Result {
        self.end(S::end)
    }
}

impl<S: ser::SerializeStruct<Ok = JsValue, Error = Error>> ser::SerializeStructVariant
    for VariantSerializer<S>
{
    type Ok = JsValue;
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<()> {
        self.inner.serialize_field(key, value)
    }

    fn end(self) -> Result {
        self.end(S::end)
    }
}

pub struct ArraySerializer<'s> {
    serializer: &'s Serializer,
    target: Array,
}

impl<'s> ArraySerializer<'s> {
    pub fn new(serializer: &'s Serializer) -> Self {
        Self {
            serializer,
            target: Array::new(),
        }
    }
}

impl ser::SerializeSeq for ArraySerializer<'_> {
    type Ok = JsValue;
    type Error = Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
        self.target.push(&value.serialize(self.serializer)?);
        Ok(())
    }

    fn end(self) -> Result {
        Ok(self.target.into())
    }
}

impl ser::SerializeTuple for ArraySerializer<'_> {
    type Ok = JsValue;
    type Error = Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result {
        ser::SerializeSeq::end(self)
    }
}

impl ser::SerializeTupleStruct for ArraySerializer<'_> {
    type Ok = JsValue;
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
        ser::SerializeTuple::serialize_element(self, value)
    }

    fn end(self) -> Result {
        ser::SerializeTuple::end(self)
    }
}

pub enum MapResult {
    Map(Map),
    Object(Object),
}

pub struct MapSerializer<'s> {
    serializer: &'s Serializer,
    target: MapResult,
    next_key: Option<JsValue>,
}

impl<'s> MapSerializer<'s> {
    pub fn new(serializer: &'s Serializer, as_object: bool) -> Self {
        Self {
            serializer,
            target: if as_object {
                MapResult::Object(Object::new())
            } else {
                MapResult::Map(Map::new())
            },
            next_key: None,
        }
    }
}

impl ser::SerializeMap for MapSerializer<'_> {
    type Ok = JsValue;
    type Error = Error;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<()> {
        debug_assert!(self.next_key.is_none());
        let next_key = key.serialize(self.serializer)?;
        if let MapResult::Object(_) = self.target {
            if !next_key.is_string() {
                return Err(Error::custom(
                    "Map key is not a string and cannot be an object key",
                ));
            }
        }

        self.next_key = Some(next_key);
        Ok(())
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
        match self.target {
            MapResult::Map(ref map) => {
                map.set(
                    &self.next_key.take().unwrap(),
                    &value.serialize(self.serializer)?,
                );
            }
            MapResult::Object(ref object) => object.set(
                self.next_key.take().unwrap(),
                value.serialize(self.serializer)?,
            ),
        }
        Ok(())
    }

    fn end(self) -> Result {
        debug_assert!(self.next_key.is_none());
        match self.target {
            MapResult::Map(map) => Ok(map.into()),
            MapResult::Object(object) => Ok(object.into()),
        }
    }
}

pub struct ObjectSerializer<'s> {
    serializer: &'s Serializer,
    target: Object,
}

impl<'s> ObjectSerializer<'s> {
    pub fn new(serializer: &'s Serializer) -> Self {
        Self {
            serializer,
            target: Object::new(),
        }
    }
}

impl ser::SerializeStruct for ObjectSerializer<'_> {
    type Ok = JsValue;
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<()> {
        let value = value.serialize(self.serializer)?;
        self.target.set(static_str_to_js(key), value);
        Ok(())
    }

    fn end(self) -> Result {
        Ok(self.target.into())
    }
}

/// A [`serde::Serializer`] that converts supported Rust values into a [`JsValue`].
// Serializer might be configurable in the future, so add but hide its implementation details.
#[derive(Default)]
pub struct Serializer {
    serialize_maps_as_objects: bool,
}

impl Serializer {
    /// Creates a new default [`Serializer`].
    pub fn new() -> Self {
        Default::default()
    }

    /// Set to `true` to serialize maps into plain JavaScript objects instead of
    /// ES2015 `Map`s. False by default.
    pub fn serialize_maps_as_objects(mut self, value: bool) -> Self {
        self.serialize_maps_as_objects = value;
        self
    }
}

macro_rules! forward_to_into {
    ($($name:ident($ty:ty);)*) => {
        $(fn $name(self, v: $ty) -> Result {
            Ok(v.into())
        })*
    };
}

impl<'s> ser::Serializer for &'s Serializer {
    type Ok = JsValue;
    type Error = Error;

    type SerializeSeq = ArraySerializer<'s>;
    type SerializeTuple = ArraySerializer<'s>;
    type SerializeTupleStruct = ArraySerializer<'s>;
    type SerializeTupleVariant = VariantSerializer<ArraySerializer<'s>>;
    type SerializeMap = MapSerializer<'s>;
    type SerializeStruct = ObjectSerializer<'s>;
    type SerializeStructVariant = VariantSerializer<ObjectSerializer<'s>>;

    forward_to_into! {
        serialize_bool(bool);

        serialize_i8(i8);
        serialize_i16(i16);
        serialize_i32(i32);

        serialize_u8(u8);
        serialize_u16(u16);
        serialize_u32(u32);

        serialize_f32(f32);
        serialize_f64(f64);

        serialize_str(&str);
    }

    // TODO: we might want to support `BigInt` here in the future.
    fn serialize_i64(self, v: i64) -> Result {
        const MAX_SAFE_INTEGER: i64 = 9_007_199_254_740_991;
        const MIN_SAFE_INTEGER: i64 = -MAX_SAFE_INTEGER;

        if v >= MIN_SAFE_INTEGER && v <= MAX_SAFE_INTEGER {
            self.serialize_f64(v as _)
        } else {
            Err(Error::custom(format_args!(
                "{} can't be represented as a JavaScript number",
                v
            )))
        }
    }

    // TODO: we might want to support `BigInt` here in the future.
    fn serialize_u64(self, v: u64) -> Result {
        const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

        if v <= MAX_SAFE_INTEGER {
            self.serialize_f64(v as _)
        } else {
            Err(Error::custom(format_args!(
                "{} can't be represented as a JavaScript number",
                v
            )))
        }
    }

    fn serialize_char(self, v: char) -> Result {
        Ok(JsString::from(v).into())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result {
        // Create a `Uint8Array` view into a Rust slice, and immediately copy it to the JS memory.
        //
        // This is necessary because any allocation in WebAssembly can require reallocation of the
        // backing memory, which will invalidate existing views (including `Uint8Array`).
        Ok(JsValue::from(Uint8Array::new(
            unsafe { Uint8Array::view(v) }.as_ref(),
        )))
    }

    fn serialize_none(self) -> Result {
        Ok(JsValue::UNDEFINED)
    }

    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result {
        Ok(JsValue::UNDEFINED)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result {
        self.serialize_unit()
    }

    /// For compatibility with serde-json, serialises unit variants as "Variant" strings.
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result {
        Ok(static_str_to_js(variant))
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result {
        VariantSerializer::new(variant, self.serialize_newtype_struct(variant, value)?).end(Ok)
    }

    /// Serialises any Rust iterable into a JS Array.
    // TODO: Figure out if there is a way to detect and serialise `Set` differently.
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Ok(ArraySerializer::new(self))
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.serialize_tuple(len)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Ok(VariantSerializer::new(
            variant,
            self.serialize_tuple_struct(variant, len)?,
        ))
    }

    /// Serialises Rust maps into JS `Map` or plain JS objects, depending on configuration of `serialize_maps_as_objects`.
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Ok(MapSerializer::new(self, self.serialize_maps_as_objects))
    }

    /// Serialises Rust typed structs into plain JS objects.
    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Ok(ObjectSerializer::new(self))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Ok(VariantSerializer::new(
            variant,
            self.serialize_struct(variant, len)?,
        ))
    }
}
