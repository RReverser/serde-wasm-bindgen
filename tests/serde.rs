use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::{from_value, to_value};
use std::{hash::Hash, collections::{HashMap, HashSet}};
use std::fmt::Debug;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_test::*;
use js_sys::{Object, Map, Reflect};

fn test<L, R>(lhs: L, rhs: R)
where
    L: Serialize + DeserializeOwned + PartialEq + Debug,
    R: Into<JsValue>,
{
    let lhs_value = to_value(&lhs).unwrap();
    assert_eq!(lhs_value, rhs.into(), "to_value from {:?}", lhs);
    let restored_lhs = from_value(lhs_value.clone()).unwrap();
    assert_eq!(lhs, restored_lhs, "from_value from {:?}", lhs_value);
}

fn test_primitive<T>(value: T)
where
    T: Copy + Serialize + Into<JsValue> + DeserializeOwned + PartialEq + Debug,
{
    test(value, value);
}

fn assert_json<R>(lhs_value: JsValue, rhs: R)
where
    R: Serialize + DeserializeOwned + PartialEq + Debug,
{
    if lhs_value.is_undefined() {
        assert_eq!(
            "null",
            serde_json::to_string(&rhs).unwrap()
        )
    } else {
        assert_eq!(
            js_sys::JSON::stringify(&lhs_value).unwrap(),
            serde_json::to_string(&rhs).unwrap(),
        );
    }

    let restored_lhs: R = from_value(lhs_value.clone()).unwrap();
    assert_eq!(restored_lhs, rhs, "from_value from {:?}", lhs_value);
}

fn test_via_json<T>(value: T)
where
    T: Serialize + DeserializeOwned + PartialEq + Debug,
{
    assert_json(to_value(&value).unwrap(), value);
}

fn assert_js_obj_eq(a: &Object, b: &Object) {
    fn keys(obj: &Object) -> Vec<JsValue> {
        if Map::instanceof(obj) {
            obj.clone().unchecked_into::<Map>().keys().into_iter().map(|key| key.unwrap()).collect()
        } else {
            Object::keys(obj).values().into_iter().map(|key| key.unwrap()).collect()
        }
    }
    
    fn get(obj: &Object, key: &JsValue) -> JsValue {
        if Map::instanceof(obj) {
            obj.clone().unchecked_into::<Map>().get(key)
        } else {
            Reflect::get(obj, key).unwrap()
        }
    }

    for key in keys(a) {
        let a_val = get(a, &key);
        let b_val = get(b, &key);

        assert_js_val_eq(&a_val, &b_val);
    }

    for key in keys(b) {
        let a_val = get(a, &key);
        let b_val = get(b, &key);

        assert_js_val_eq(&a_val, &b_val);
    }
}

fn assert_js_val_eq(a: &JsValue, b: &JsValue) {
    if a.is_object() {
        assert!(b.is_object());
        assert_js_obj_eq(a.dyn_ref::<Object>().unwrap(), b.dyn_ref::<Object>().unwrap());
    } else if a.as_f64().is_some() {
        assert!(b.as_f64().is_some());
        assert_eq!(a.as_f64().unwrap(), b.as_f64().unwrap());
    } else if a.is_string() {
        assert!(b.is_string());
        assert_eq!(a.as_string().unwrap(), b.as_string().unwrap());
    } else if a.is_undefined() {
        assert!(b.is_undefined());
    } else {
        todo!()
    }
}

fn assert_via_js_val<R>(lhs_value: JsValue, rhs: R)
where
    R: Serialize + DeserializeOwned + PartialEq + Debug,
{
    let restored_lhs: R = from_value(lhs_value.clone()).unwrap();
    assert_eq!(restored_lhs, rhs, "from_value from {:?}", lhs_value);
}

fn test_via_js_val<T>(value: T)
where
    T: Serialize + DeserializeOwned + PartialEq + Debug,
{
    assert_via_js_val(to_value(&value).unwrap(), value);
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
        enum $name<A, B> where A: Debug + Hash + Eq {
            Unit,
            Newtype(A),
            Tuple(A, B),
            Struct { a: A, b: B },
            Map(HashMap<A, B>),
            Seq { seq: Vec<B> } // internal tags cannot be directly embedded in arrays
        }

        test_via_json($name::Unit::<String, i32>);
        test_via_json($name::Newtype::<_, i32>("newtype content".to_string()));
        test_via_json($name::Tuple("tuple content".to_string(), 42));
        test_via_json($name::Struct {
            a: "struct content".to_string(),
            b: 42,
        });
        test_via_js_val($name::Map::<String, i32>(
            vec![
                ("a".to_string(), 12), 
                ("abc".to_string(), -1161), 
                ("b".to_string(), 64)
            ].into_iter().collect()
        ));
        test_via_js_val($name::Map::<i32, i32>(
            vec![
                (54, 12), 
                (2, -1161), 
                (-51, 64)
            ].into_iter().collect()
        ));
        test_via_js_val($name::Seq::<i32, f64> { seq: vec![5.4, 63.1, 0.2, -62.12, 6.0] });
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
    fn test_str(s: &'static str) {
        let value = to_value(&s).unwrap();
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
    test(Some(0_u32), 0_u32);
    test(Some(32_u32), 32_u32);
    test(None::<u32>, JsValue::UNDEFINED);

    test(Some("".to_string()), "");
    test(Some("abc".to_string()), "abc");
    test(None::<String>, JsValue::UNDEFINED);

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

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    #[serde(tag = "tag")]
    enum InternallyTagged<A, B> {
        Unit,
        Struct { a: A, b: B },
        Sequence { seq: Vec<A> }
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
    test_via_json(InternallyTagged::<i32, ()>::Sequence { seq: vec![12, 41, -11, -65, 961] });

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
fn test_externally_tagged_enum() {
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    enum Test {
        Seq(Vec<f64>),
        Map(HashMap<String, i32>),
    }

    assert_js_val_eq(
        &to_value(&Test::Seq(vec![5.4, 63.1, 0.2, -62.12, 6.0])).unwrap(),
        &js_sys::eval(r#"({ "Seq": [5.4, 63.1, 0.2, -62.12, 6.0] })"#).unwrap()
    );

    let map = Test::Map(
        vec![
            ("a".to_string(), 12), 
            ("abc".to_string(), -1161), 
            ("b".to_string(), 64)
        ].into_iter().collect()
    );
    let js_map = to_value(&map).unwrap();
    assert_js_val_eq(
        &js_map,
        &js_sys::eval(r#"({ "Map": { "a": 12, "abc": -1161, "b": 64 } })"#).unwrap()
    );
    assert_eq!(&map, &from_value::<Test>(js_map).unwrap());
}

#[wasm_bindgen_test]
fn test_internally_tagged_enum() {
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    #[serde(tag = "kind")]
    enum Test {
        Seq { seq: Vec<f64> },
        Map(HashMap<String, i32>),
    }

    let seq = Test::Seq { seq: vec![5.4, 63.1, 0.2, -62.12, 6.0] };
    let js_seq = to_value(&seq).unwrap();
    assert_js_val_eq(
        &js_seq,
        &js_sys::eval(r#"({ kind: "Seq", seq: [5.4, 63.1, 0.2, -62.12, 6.0] })"#).unwrap()
    );
    assert_eq!(&seq, &from_value::<Test>(js_seq).unwrap());

    let map = Test::Map(
        vec![
            ("a".to_string(), 12), 
            ("abc".to_string(), -1161), 
            ("b".to_string(), 64)
        ].into_iter().collect()
    );
    let js_map = to_value(&map).unwrap();
    assert_js_val_eq(
        &js_map,
        &js_sys::eval(r#"({ "kind": "Map", "a": 12, "abc": -1161, "b": 64 })"#).unwrap()
    );
    assert_eq!(&map, &from_value::<Test>(js_map).unwrap());
}

#[wasm_bindgen_test]
fn test_adjacently_tagged_enum() {
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    #[serde(tag = "kind", content = "content")]
    enum Test {
        Seq(Vec<f64>),
        Map(HashMap<String, i32>),
    }

    let seq = Test::Seq(vec![5.4, 63.1, 0.2, -62.12, 6.0]);
    let js_seq = to_value(&seq).unwrap();
    assert_js_val_eq(
        &js_seq,
        &js_sys::eval(r#"({ kind: "Seq", content: [5.4, 63.1, 0.2, -62.12, 6.0] })"#).unwrap()
    );
    assert_eq!(&seq, &from_value::<Test>(js_seq).unwrap());

    let map = Test::Map(
        vec![
            ("a".to_string(), 12), 
            ("abc".to_string(), -1161), 
            ("b".to_string(), 64)
        ].into_iter().collect()
    );
    let js_map = to_value(&map).unwrap();
    assert_js_val_eq(
        &js_map,
        &js_sys::eval(r#"({ kind: "Map", content: { "a": 12, "abc": -1161, "b": 64 } })"#).unwrap()
    );
    assert_eq!(&map, &from_value::<Test>(js_map).unwrap());
}

#[wasm_bindgen_test]
fn structs() {
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Unit;

    test(Unit, JsValue::UNDEFINED);

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
    test_via_json::<HashSet<bool>>([false, true].iter().cloned().collect());
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
    let mut src = HashMap::new();

    src.insert(
        Struct {
            a: 1,
            b: "smth".to_string(),
        },
        Struct {
            a: 2,
            b: "SMTH".to_string(),
        },
    );

    src.insert(
        Struct {
            a: 42,
            b: "something".to_string(),
        },
        Struct {
            a: 84,
            b: "SOMETHING".to_string(),
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
