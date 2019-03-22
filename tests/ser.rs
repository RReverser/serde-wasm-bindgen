use serde::Serialize;
use serde_wasm_bindgen::to_value;
use std::collections::{HashMap, HashSet};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_test::*;

fn test<L: Serialize, R: std::fmt::Debug>(lhs: L, rhs: R)
where
    JsValue: PartialEq<R>,
{
    assert_eq!(to_value(&lhs).unwrap(), rhs);
}

fn test_primitive<T: Copy + Serialize + std::fmt::Debug>(value: T)
where
    JsValue: PartialEq<T>,
{
    test(value, value);
}

fn assert_json<R: Serialize>(lhs: JsValue, rhs: R) {
    assert_eq!(
        js_sys::JSON::stringify(&lhs).unwrap(),
        serde_json::to_string(&rhs).unwrap(),
    );
}

fn test_via_json<T: Serialize>(value: T) {
    assert_json(to_value(&value).unwrap(), value);
}

macro_rules! test_unsigned {
    ($ty:ident) => {{
        test_primitive::<$ty>(0 as _);
        test_primitive::<$ty>(42 as _);
        test_primitive::<$ty>(std::$ty::MIN);
        test_primitive::<$ty>(std::$ty::MAX);
    }};
}

macro_rules! test_signed {
    ($ty:ident) => {{
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
        #[derive(Serialize)]
        enum $name<A, B> {
            Unit,
            Newtype(A),
            Tuple(A, B),
            Struct { a: A, b: B },
        }

        test_via_json($name::Unit::<(), ()>);
        test_via_json($name::Newtype::<_, ()>("newtype content"));
        test_via_json($name::Tuple("tuple content", 42));
        test_via_json($name::Struct {
            a: "struct content",
            b: 42,
        });
    }};
}

#[wasm_bindgen_test]
fn unit() {
    test((), JsValue::UNDEFINED);
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

    {
        const MAX_SAFE_INTEGER: i64 = 9_007_199_254_740_991;

        test(0_i64, 0_f64);
        test(42_i64, 42_f64);
        test(-42_i64, -42_f64);
        test(MAX_SAFE_INTEGER, MAX_SAFE_INTEGER as f64);
        test(-MAX_SAFE_INTEGER, -MAX_SAFE_INTEGER as f64);
        to_value(&(MAX_SAFE_INTEGER + 1)).unwrap_err();
        to_value(&-(MAX_SAFE_INTEGER + 1)).unwrap_err();
        to_value(&std::i64::MIN).unwrap_err();
        to_value(&std::i64::MAX).unwrap_err();
    }

    {
        const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

        test(0_u64, 0_f64);
        test(42_u64, 42_f64);
        test(MAX_SAFE_INTEGER, MAX_SAFE_INTEGER as f64);
        to_value(&(MAX_SAFE_INTEGER + 1)).unwrap_err();
        to_value(&std::u64::MAX).unwrap_err();
    }

    test_float!(f32);
    test_float!(f64);
}

#[wasm_bindgen_test]
fn strings() {
    test_primitive("");
    test_primitive("abc");
    test_primitive("\0");
    test_primitive("😃");
}

#[wasm_bindgen_test]
fn chars() {
    test('a', "a");
    test('\0', "\0");
    test('😃', "😃");
}

#[wasm_bindgen_test]
fn bytes() {
    // Create a backing storage.
    let mut src = [1, 2, 3];
    // Store the original separately for the mutation test
    let orig_src = src;
    // Convert to a JS value
    let res = to_value(&serde_bytes::Bytes::new(&src)).unwrap();
    // Modify the original storage to make sure that JS value is a copy.
    src[0] = 10;
    // Make sure the JS value is a Uint8Array
    let res = res.dyn_into::<js_sys::Uint8Array>().unwrap();
    // Copy it into another Rust storage
    let mut dst = [0; 3];
    res.copy_to(&mut dst);
    // Finally, compare that resulting storage with the original.
    assert_eq!(orig_src, dst);
}

#[wasm_bindgen_test]
fn options() {
    test(Some(0_u32), 0_u32);
    test(Some(32_u32), 32_u32);
    test(None::<u32>, JsValue::UNDEFINED);

    test(Some(""), "");
    test(Some("abc"), "abc");
    test(None::<&str>, JsValue::UNDEFINED);

    // This one is an unfortunate edge case, but not very likely in real world.
    test(Some(()), JsValue::UNDEFINED);
    test(None::<()>, JsValue::UNDEFINED);
    test(Some(Some(())), JsValue::UNDEFINED);
    test(Some(None::<()>), JsValue::UNDEFINED);
}

#[wasm_bindgen_test]
fn enums() {
    test_enum! {
        ExternallyTagged
    }
    test_enum! {
        #[serde(tag = "tag")]
        InternallyTagged
    }
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
    #[derive(Serialize)]
    struct Unit;

    test(Unit, JsValue::UNDEFINED);

    #[derive(Serialize)]
    struct Newtype<A>(A);

    test_via_json(Newtype("newtype content"));

    #[derive(Serialize)]
    struct Tuple<A, B>(A, B);

    test_via_json(Tuple("tuple content", 42));

    #[derive(Serialize)]
    struct Struct<A, B> {
        a: A,
        b: B,
    }

    test_via_json(Struct {
        a: "struct content",
        b: 42,
    });
}

#[wasm_bindgen_test]
fn sequences() {
    test_via_json([1, 2]);
    test_via_json(["", "x", "xyz"]);
    test_via_json((100, "xyz", true));

    // Sets are currently indistinguishable from other sequences for
    // Serde serialisers, so this will become an array on the JS side.
    test_via_json::<HashSet<bool>>([false, true].iter().cloned().collect());
}

#[wasm_bindgen_test]
fn maps() {
    #[derive(Serialize, PartialEq, Eq, Hash)]
    struct Struct<A, B> {
        a: A,
        b: B,
    }

    // Create a Rust HashMap with non-string keys to make sure
    // that we support real arbitrary maps.
    let mut src = HashMap::new();

    src.insert(Struct { a: 1, b: "smth" }, Struct { a: 2, b: "SMTH" });

    src.insert(
        Struct {
            a: 42,
            b: "something",
        },
        Struct {
            a: 84,
            b: "SOMETHING",
        },
    );

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
