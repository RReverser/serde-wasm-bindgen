# `serde-wasm-bindgen`

This is an alternative native integration of [Serde] with [wasm-bindgen].

[serde]: https://serde.rs
[wasm-bindgen]: https://github.com/rustwasm/wasm-bindgen

## Motivation

This library was created to address [wasm-bindgen#1258] and provide a native
Serde integration for wasm-bindgen to directly convert values between JavaScript
and Rust (compiled to WebAssembly).

The primary difference with the [built-in implementation] is that it leverages
direct APIs for JavaScript value manipulation instead of passing around
stringified JSON data. This allows it to support more types while producing a
much leaner Wasm binary. In particular, it saved 26.6KB when comparing
size-optimised and Brotli-compressed [benchmarks] with stripped debug
information.

Performance-wise the library is currently comparable with the original. Specific
numbers vary a lot between the engines and used data types and, according to
benchmarks, range from 1.6x regression in worst cases to 3.3x improvement in
best cases. Your mileage might vary.

These numbers are currently mostly saturated by the overhead of frequent
JavaScript <-> Wasm and JavaScript <-> C++ calls. These calls are used for
sharing JavaScript values with the Rust side as well as encoding/decoding UTF-8
strings, and will go away in the future when [reference types] proposal lands
natively in Wasm.

[wasm-bindgen#1258]: https://github.com/rustwasm/wasm-bindgen/issues/1258
[built-in implementation]: https://rustwasm.github.io/docs/wasm-bindgen/reference/arbitrary-data-with-serde.html
[benchmarks]: benchmarks
[reference types]: https://github.com/WebAssembly/reference-types

## Usage

To pass a Rust value to JavaScript, use:

```rust
use wasm_bindgen::JsValue;
use serde::Serialize;
use serde_wasm_bindgen as swb;

#[derive(Serialize)]
struct Foo {
  num: usize,
}

pub fn pass_value_to_js() -> Result<JsValue, swb::Error> {
  let foo = Foo { num: 37 };
  swb::to_value(&foo)
}
```

Likewise, the `from_value` function can be used for deserialization.

## Supported Types

Note that, even though it might often be the case, this library doesn't attempt
to be strictly compatible with either [`serde_json`][serde_json] or,
correspondingly, `JsValue::from_serde` / `JsValue::into_serde`, instead
prioritising better compatibility with common JavaScript idioms and
representations.

[serde_json]: https://docs.serde.rs/serde_json/

### Deserialization

| Javascript Type                            | Rust Type                          |
| ------------------------------------------ | ---------------------------------- |
| `undefined` and `null`                     | `()` or `Option<T>`                |
| `false` and `true`                         | `bool`                             |
| Any [safe integer]                         | `u8`/`i8`/.../`u128`/`i128`        |
| JS Number                                  | `f32` or `f64`                     |
| Length-1 String                            | `char`                             |
| String                                     | `String` or `Cow<'static, str>`    |
| `"Variant"`                                | Enum Variant<sup>†</sup>           |
| `{ key1: value1, ... }` Object             | `HashMap<String, T>` or Struct `T` |
| Any Iterable of `[key, value]`<sup>‡</sup> | `HashMap`, `BTreeMap`, etc.        |
| Any Iterable of `value`                    | Tuple, `Vec`, `HashSet`, etc.      |
| `ArrayBuffer` or `Uint8Array`              | [serde_bytes] Byte Buffer          |

**Notes:**

- †: The specific representation is [controlled] by `#[serde(...)]` attributes
  and should be compatible with `serde-json`.
- ‡: Excepts are [internally tagged] and [untagged] enums. These representations
  currently do not support deserializing map-like iterables. They only support
  deserialization from `Object` due to their special treatment in `serde`. This
  restriction may be lifted at some point in the future if a `serde(with = ...)`
  attribute can define the expected Javascript representation of the variant, or
  if [serde-rs/serde#1183] gets resolved.

[safe integer]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number/isSafeInteger
[internally tagged]: https://serde.rs/enum-representations.html#internally-tagged
[untagged]: https://serde.rs/enum-representations.html#untagged
[serde_bytes]: https://github.com/serde-rs/bytes
[controlled]: https://serde.rs/enum-representations.html
[serde-rs/serde#1183]: https://github.com/serde-rs/serde/issues/1183

### Serialization

Serialization is mostly the same as deserialization, but is limited to a single
representation, so it chooses:

| Rust Type       | Javascript Type          |
| --------------- | ------------------------ |
| `None`          | `undefined`              |
| `()`            | `undefined`              |
| `HashMap`, etc. | ES2015 `Map`<sup>†</sup> |
| `Vec`, etc.     | `Array`                  |
| Byte Buffers    | `Uint8Array`             |
| Struct `T`      | Object                   |

**Notes:**

- †: You can use `serialize_maps_as_objects(true)` to alter this behaviour. But
  keep in mind that JS Object keys are always stored as `String`, even if the
  original Rust type had integer keys!

## License

Licensed under the MIT license. See the [LICENSE](LICENSE) file for details.
