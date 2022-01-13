use js_sys::BigInt;
use maplit::{btreemap, hashmap, hashset};
use serde::de::DeserializeOwned;
use serde::ser::Error as SerError;
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::{from_value, to_value, Error, Serializer};
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::hash::Hash;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

fn test_via_into<L, R>(lhs: L, rhs: R)
where
    L: Serialize + DeserializeOwned + PartialEq + Debug,
    R: Into<JsValue>,
{
    test_via_into_with_config(lhs, rhs, &Serializer::new())
}

fn test_via_into_with_config<L, R>(lhs: L, rhs: R, serializer: &Serializer)
where
    L: Serialize + DeserializeOwned + PartialEq + Debug,
    R: Into<JsValue>,
{
    let lhs_value = lhs.serialize(serializer).unwrap();
    assert_eq!(lhs_value, rhs.into(), "to_value from {:?}", lhs);
    let restored_lhs = from_value(lhs_value.clone()).unwrap();
    assert_eq!(lhs, restored_lhs, "from_value from {:?}", lhs_value);
}

fn test_primitive<T>(value: T)
where
    T: Copy + Serialize + Into<JsValue> + DeserializeOwned + PartialEq + Debug,
{
    test_via_into(value, value);
}

fn assert_json<R>(lhs_value: JsValue, rhs: R)
where
    R: Serialize + DeserializeOwned + PartialEq + Debug,
{
    if lhs_value.is_undefined() {
        assert_eq!("null", serde_json::to_string(&rhs).unwrap())
    } else {
        assert_eq!(
            js_sys::JSON::stringify(&lhs_value).unwrap(),
            serde_json::to_string(&rhs).unwrap(),
        );
    }

    let restored_lhs: R = from_value(lhs_value.clone()).unwrap();
    assert_eq!(restored_lhs, rhs, "from_value from {:?}", lhs_value);
}

fn test_via_json_with_config<T>(value: T, serializer: Serializer)
where
    T: Serialize + DeserializeOwned + PartialEq + Debug,
{
    assert_json(value.serialize(&serializer).unwrap(), value);
}

fn test_via_json<T>(value: T)
where
    T: Serialize + DeserializeOwned + PartialEq + Debug,
{
    test_via_json_with_config(value, Serializer::new());
}

fn test_via_round_trip<T>(value: T, serializer: Serializer)
where
    T: Serialize + DeserializeOwned + PartialEq + Debug + Clone,
{
    let original = value.clone();
    let serialized = value.serialize(&serializer).unwrap();
    let round_trip = from_value(serialized).unwrap();
    assert_eq!(original, round_trip);
}

macro_rules! test_unsigned {
    ($ty:ident) => {{
        test_primitive::<$ty>(42 as _);
        test_primitive::<$ty>(std::$ty::MIN);
        test_primitive::<$ty>(std::$ty::MAX);
    }};
}

macro_rules! test_signed {
    ($ty:ident) => {{
        test_primitive::<$ty>(0 as _);
        test_primitive::<$ty>(-42 as _);
        test_unsigned!($ty);
    }};
}

macro_rules! test_float {
    ($ty:ident) => {{
        test_primitive::<$ty>(0.42);
        test_primitive::<$ty>(-0.42);
        test_signed!($ty);
        test_primitive::<$ty>(std::$ty::EPSILON);
        test_primitive::<$ty>(std::$ty::MIN_POSITIVE);
        assert!(match to_value::<$ty>(&std::$ty::NAN).unwrap().as_f64() {
            Some(v) => v.is_nan(),
            None => false,
        });
        test_primitive::<$ty>(std::$ty::INFINITY);
        test_primitive::<$ty>(std::$ty::NEG_INFINITY);
    }};
}

macro_rules! test_enum {
    ($(# $attr:tt)* $name:ident) => {{
        #[derive(Debug, PartialEq, Serialize, Deserialize)]
        $(# $attr)*
        enum $name<A, B> where A: Debug + Ord + Eq {
            Unit,
            Newtype(A),
            Tuple(A, B),
            Struct { a: A, b: B },
            Map(BTreeMap<A, B>),
            Seq { seq: Vec<B> } // internal tags cannot be directly embedded in arrays
        }

        test_via_json($name::Unit::<String, i32>);
        test_via_json($name::Newtype::<_, i32>("newtype content".to_string()));
        test_via_json($name::Tuple("tuple content".to_string(), 42));
        test_via_json($name::Struct {
            a: "struct content".to_string(),
            b: 42,
        });
        test_via_json_with_config($name::Map::<String, i32>(
            btreemap!{
                "a".to_string() => 12,
                "abc".to_string() => -1161,
                "b".to_string() => 64,
            }
        ), Serializer::new().serialize_maps_as_objects(true));
        test_via_json($name::Seq::<i32, i32> { seq: vec![5, 63, 0, -62, 6] });
    }};
}

#[wasm_bindgen_test]
fn unit() {
    test_via_into((), JsValue::UNDEFINED);
}

#[wasm_bindgen_test]
fn bool() {
    test_primitive(false);
    test_primitive(true);
}

#[wasm_bindgen_test]
fn numbers() {
    test_signed!(i8);
    test_unsigned!(u8);

    test_signed!(i16);
    test_unsigned!(u16);

    test_signed!(i32);
    test_unsigned!(u32);

    test_float!(f32);
    test_float!(f64);

    {
        const MAX_SAFE_INTEGER: i64 = 9_007_199_254_740_991;

        test_via_into(0_i64, 0_f64);
        test_via_into(42_i64, 42_f64);
        test_via_into(-42_i64, -42_f64);
        test_via_into(MAX_SAFE_INTEGER, MAX_SAFE_INTEGER as f64);
        test_via_into(-MAX_SAFE_INTEGER, -MAX_SAFE_INTEGER as f64);
        to_value(&(MAX_SAFE_INTEGER + 1)).unwrap_err();
        to_value(&-(MAX_SAFE_INTEGER + 1)).unwrap_err();
        to_value(&std::i64::MIN).unwrap_err();
        to_value(&std::i64::MAX).unwrap_err();
    }

    {
        const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

        test_via_into(0_u64, 0_f64);
        test_via_into(42_u64, 42_f64);
        test_via_into(MAX_SAFE_INTEGER, MAX_SAFE_INTEGER as f64);
        to_value(&(MAX_SAFE_INTEGER + 1)).unwrap_err();
        to_value(&std::u64::MAX).unwrap_err();
    }

    // By default serializing i128 and u128 results in an error
    {
        to_value(&0_i128).unwrap_err();
        to_value(&0_u128).unwrap_err();
    }

    // By default deserializing i128 and u128 uses 64 bit implementation
    {
        assert_eq!(from_value::<i128>(JsValue::from(0_i128)).unwrap(), 0);
        assert_eq!(from_value::<i128>(JsValue::from(42_i128)).unwrap(), 42);
        assert_eq!(from_value::<i128>(JsValue::from(-42_i128)).unwrap(), -42);
        assert_eq!(from_value::<u128>(JsValue::from(0_u128)).unwrap(), 0);
        assert_eq!(from_value::<u128>(JsValue::from(42_u128)).unwrap(), 42);
    }

    // Test large number bigint serialization feature
    let bigint_serializer = Serializer::new().serialize_large_number_types_as_bigints(true);
    {
        const MAX_SAFE_INTEGER: i64 = 9_007_199_254_740_991;

        // Should be bigint
        assert!(0_i64.serialize(&bigint_serializer).unwrap().is_bigint());

        // u64 and i64 should serialize the same
        test_via_into_with_config(0_i64, 0_u64, &bigint_serializer);
        test_via_into_with_config(42_i64, 42_u64, &bigint_serializer);

        // Js-numbers should also deserialize into 64 bit types
        assert_eq!(from_value::<i64>(JsValue::from_f64(1.0)).unwrap(), 1);
        assert_eq!(from_value::<i64>(JsValue::from_f64(-1.0)).unwrap(), -1);

        // Invalid floats should fail
        from_value::<i64>(JsValue::from_f64(1.5)).unwrap_err();
        from_value::<i64>(JsValue::from_f64(-10.2)).unwrap_err();

        // Big ints that are too large or small should error
        from_value::<i64>(BigInt::from(i128::MAX).into()).unwrap_err();
        from_value::<i64>(BigInt::from(i128::MIN).into()).unwrap_err();

        // Test near max safe float
        assert_eq!(
            (MAX_SAFE_INTEGER + 1)
                .serialize(&bigint_serializer)
                .unwrap(),
            MAX_SAFE_INTEGER + 1
        );
        assert_eq!(
            (-(MAX_SAFE_INTEGER + 1))
                .serialize(&bigint_serializer)
                .unwrap(),
            -(MAX_SAFE_INTEGER + 1)
        );

        // Handle extreme values
        assert_eq!(
            std::i64::MIN.serialize(&bigint_serializer).unwrap(),
            JsValue::from(std::i64::MIN)
        );
        assert_eq!(
            std::i64::MAX.serialize(&bigint_serializer).unwrap(),
            JsValue::from(std::i64::MAX)
        );
    }

    {
        const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

        // Should be bigint
        assert!(0_u64.serialize(&bigint_serializer).unwrap().is_bigint());

        // u64 and i64 should serialize the same
        test_via_into_with_config(0_u64, 0_i64, &bigint_serializer);
        test_via_into_with_config(42_u64, 42_i64, &bigint_serializer);
        test_via_into_with_config(
            MAX_SAFE_INTEGER,
            MAX_SAFE_INTEGER as i64,
            &bigint_serializer,
        );

        // Can still deserialize from JS numbers
        assert_eq!(from_value::<u64>(JsValue::from_f64(1.0)).unwrap(), 1);

        // Invalid floats should fail
        from_value::<u64>(JsValue::from_f64(1.5)).unwrap_err();
        from_value::<u64>(JsValue::from_f64(-10.2)).unwrap_err();

        // Big ints that are too large or small should error
        from_value::<u64>(BigInt::from(i128::MAX).into()).unwrap_err();
        from_value::<u64>(BigInt::from(i128::MIN).into()).unwrap_err();

        // Test large numbers
        assert_eq!(
            (MAX_SAFE_INTEGER + 1)
                .serialize(&bigint_serializer)
                .unwrap(),
            (MAX_SAFE_INTEGER + 1)
        );
        assert_eq!(
            std::u64::MAX.serialize(&bigint_serializer).unwrap(),
            JsValue::from(std::u64::MAX)
        );
    }

    // i128 and u128 should serialize to bigint when the feature is enabled
    {
        // Should be bigint
        assert!(0_i128.serialize(&bigint_serializer).unwrap().is_bigint());

        // i128 and u128 should serialize the same
        test_via_into_with_config(0_u128, 0_i128, &bigint_serializer);
        test_via_into_with_config(42_u128, 42_i128, &bigint_serializer);

        // Can still deserialize from JS numbers
        assert_eq!(from_value::<i128>(JsValue::from_f64(1.0)).unwrap(), 1);

        // Invalid floats should fail
        from_value::<i128>(JsValue::from_f64(1.5)).unwrap_err();
        from_value::<i128>(JsValue::from_f64(-10.2)).unwrap_err();
    }

    {
        // Should be bigint
        assert!(0_u128.serialize(&bigint_serializer).unwrap().is_bigint());

        // i128 and u128 should serialize the same
        test_via_into_with_config(0_i128, 0_u128, &bigint_serializer);
        test_via_into_with_config(42_i128, 42_u128, &bigint_serializer);

        // Can still deserialize from JS numbers
        assert_eq!(from_value::<u128>(JsValue::from_f64(1.0)).unwrap(), 1);

        // Invalid floats should fail
        from_value::<u128>(JsValue::from_f64(1.5)).unwrap_err();
        from_value::<u128>(JsValue::from_f64(-10.2)).unwrap_err();
    }
}

#[wasm_bindgen_test]
fn strings() {
    fn test_str(s: &'static str) {
        let value = to_value(s).unwrap();
        assert_eq!(value, s);
        let restored: String = from_value(value).unwrap();
        assert_eq!(s, restored);
    }

    test_str("");
    test_str("abc");
    test_str("\0");
    test_str("😃");
}

#[wasm_bindgen_test]
fn chars() {
    test_via_into('a', "a");
    test_via_into('\0', "\0");
    test_via_into('😃', "😃");
}

#[wasm_bindgen_test]
fn bytes() {
    // Create a backing storage.
    let mut src = [1, 2, 3];
    // Store the original separately for the mutation test
    let orig_src = src;
    // Convert to a JS value
    let value = to_value(&serde_bytes::Bytes::new(&src)).unwrap();
    // Modify the original storage to make sure that JS value is a copy.
    src[0] = 10;

    // Make sure the JS value is a Uint8Array
    let res = value.dyn_ref::<js_sys::Uint8Array>().unwrap();
    // Copy it into another Rust storage
    let mut dst = [0; 3];
    res.copy_to(&mut dst);
    // Finally, compare that resulting storage with the original.
    assert_eq!(orig_src, dst);

    // Now, try to deserialize back.
    let deserialized: serde_bytes::ByteBuf = from_value(value).unwrap();
    assert_eq!(deserialized.as_ref(), orig_src);
}

#[wasm_bindgen_test]
fn options() {
    test_via_into(Some(0_u32), 0_u32);
    test_via_into(Some(32_u32), 32_u32);
    test_via_into(None::<u32>, JsValue::UNDEFINED);

    test_via_into(Some("".to_string()), "");
    test_via_into(Some("abc".to_string()), "abc");
    test_via_into(None::<String>, JsValue::UNDEFINED);

    // This one is an unfortunate edge case that won't roundtrip,
    // but it's pretty unlikely in real-world code.
    assert_eq!(to_value(&Some(())).unwrap(), JsValue::UNDEFINED);
    assert_eq!(to_value(&None::<()>).unwrap(), JsValue::UNDEFINED);
    assert_eq!(to_value(&Some(Some(()))).unwrap(), JsValue::UNDEFINED);
    assert_eq!(to_value(&Some(None::<()>)).unwrap(), JsValue::UNDEFINED);
}

#[wasm_bindgen_test]
fn enums() {
    test_enum! {
        ExternallyTagged
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
    #[serde(tag = "tag")]
    enum InternallyTagged<A, B>
    where
        A: Ord,
    {
        Unit,
        Struct { a: A, b: B },
        Sequence { seq: Vec<A> },
        Map(BTreeMap<A, B>),
    }

    test_via_json(InternallyTagged::Unit::<(), ()>);
    test_via_json(InternallyTagged::Struct {
        a: "struct content".to_string(),
        b: 42,
    });
    test_via_json(InternallyTagged::Struct {
        a: "struct content".to_string(),
        b: 42.2,
    });
    test_via_json(InternallyTagged::<i32, ()>::Sequence {
        seq: vec![12, 41, -11, -65, 961],
    });

    // Internal tags with maps are not properly deserialized from Map values due to the exclusion
    // of Iterables during deserialize_any(). They can be deserialized properly from plain objects
    // so we can test that.
    test_via_json_with_config(
        InternallyTagged::Map(btreemap! {
            "a".to_string() => 12,
            "abc".to_string() => -1161,
            "b".to_string() => 64,
        }),
        Serializer::new().serialize_maps_as_objects(true),
    );

    test_via_round_trip(
        InternallyTagged::Struct {
            a: 10_u64,
            b: -10_i64,
        },
        Serializer::new().serialize_large_number_types_as_bigints(true),
    );

    test_enum! {
        #[serde(tag = "tag", content = "content")]
        AdjacentlyTagged
    }
    test_enum! {
        #[serde(untagged)]
        Untagged
    }
}

#[wasm_bindgen_test]
fn structs() {
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Unit;

    test_via_into(Unit, JsValue::UNDEFINED);

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Newtype<A>(A);

    test_via_json(Newtype("newtype content".to_string()));

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Tuple<A, B>(A, B);

    test_via_json(Tuple("tuple content".to_string(), 42));

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Struct<A, B> {
        a: A,
        b: B,
    }

    test_via_json(Struct {
        a: "struct content".to_string(),
        b: 42,
    });
}

#[wasm_bindgen_test]
fn sequences() {
    test_via_json([1, 2]);
    test_via_json(["".to_string(), "x".to_string(), "xyz".to_string()]);
    test_via_json((100, "xyz".to_string(), true));

    // Sets are currently indistinguishable from other sequences for
    // Serde serialisers, so this will become an array on the JS side.
    test_via_json(hashset! {false, true});
}

#[wasm_bindgen_test]
fn maps() {
    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
    struct Struct<A, B> {
        a: A,
        b: B,
    }

    // Create a Rust HashMap with non-string keys to make sure
    // that we support real arbitrary maps.
    let src = hashmap! {
        Struct {
            a: 1,
            b: "smth".to_string(),
        } => Struct {
            a: 2,
            b: "SMTH".to_string(),
        },
        Struct {
            a: 42,
            b: "something".to_string(),
        } => Struct {
            a: 84,
            b: "SOMETHING".to_string(),
        },
    };

    // Convert to a JS value
    let res = to_value(&src).unwrap();

    // Make sure that the result is an ES6 Map.
    let res = res.dyn_into::<js_sys::Map>().unwrap();
    assert_eq!(res.size() as usize, src.len());

    // Compare values one by one (it's ok to use JSON for invidivual structs).
    res.entries()
        .into_iter()
        .map(|kv| kv.unwrap())
        .zip(src)
        .for_each(|(lhs_kv, rhs_kv)| {
            assert_json(lhs_kv, rhs_kv);
        });
}

#[wasm_bindgen_test]
fn maps_objects_string_key() {
    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
    struct Struct<A, B> {
        a: A,
        b: B,
    }

    let src = hashmap! {
        "a".to_string() => Struct {
            a: 2,
            b: "S".to_string(),
        },
        "b".to_string() => Struct {
            a: 3,
            b: "T".to_string(),
        },
    };

    test_via_json_with_config(src, Serializer::new().serialize_maps_as_objects(true));
}

#[wasm_bindgen_test]
fn maps_objects_object_key() {
    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
    struct Struct<A, B> {
        a: A,
        b: B,
    }

    let serializer = Serializer::new().serialize_maps_as_objects(true);

    let src = hashmap! {
        Struct {
            a: 1,
            b: "smth".to_string(),
        } => Struct {
            a: 2,
            b: "SMTH".to_string(),
        },
        Struct {
            a: 42,
            b: "something".to_string(),
        } => Struct {
            a: 84,
            b: "SOMETHING".to_string(),
        },
    };

    let res = src.serialize(&serializer).unwrap_err();
    assert_eq!(
        res.to_string(),
        Error::custom("Map key is not a string and cannot be an object key").to_string()
    );
}
