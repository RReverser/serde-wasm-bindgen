This is an alternative native integration of [Serde](https://serde.rs/) with [wasm-bindgen](https://github.com/rustwasm/wasm-bindgen).

## Why

This library was created to address [rustwasm/wasm-bindgen#1258](https://github.com/rustwasm/wasm-bindgen/issues/1258) and provide a native Serde integration for wasm-bindgen to directly convert values between JavaScript and Rust (compiled to WebAssembly).

The primary difference with the [built-in implementation](https://rustwasm.github.io/docs/wasm-bindgen/reference/arbitrary-data-with-serde.html) is that it leverages direct APIs for JavaScript value manipulation instead of passing data in a JSON format. This allows it to support more types while producing a much leaner Wasm binary. In particular, it saved 26.6KB when comparing size-optimised and Brotli-compressed [benchmarks](benchmarks) with a stripped debug information.

Performance-wise the library is currently comparable with the original. Specific numbers vary a lot between the engines and used data types and, according to benchmarks, range from 1.6x regression in worst cases to 3.3x improvement in best cases. Your mileage might vary.

These numbers are currently mostly saturated by the overhead of frequent JavaScript <-> Wasm and JavaScript <-> C++ calls. These calls are used for sharing JavaScript values with the Rust side as well as encoding/decoding UTF-8 strings, and will go away in the future when [reference types](https://github.com/WebAssembly/reference-types) proposal lands natively in Wasm.

## Usage

To pass a Rust value to JavaScript, use:

```rust
#[wasm_bindgen]
pub fn pass_value_to_js() -> Result<JsValue, JsValue> {
	// ...
	serde_wasm_bindgen::to_value(&some_supported_rust_value)
}
```

To retrieve a value from JavaScript:

```rust
#[wasm_bindgen]
pub fn get_value_from_js(value: JsValue) -> Result<(), JsValue> {
	let value: SomeSupportedRustType = serde_wasm_bindgen::from_value(value)?;
	// ...
	Ok(())
}
```

## Supported types

Note that, even though it might often be the case, this library doesn't attempt to be strictly compatible with either [`serde_json`](https://docs.serde.rs/serde_json/) or, correspondingly, `JsValue::from_serde` / `JsValue::into_serde`, instead prioritising better compatibility with common JavaScript idioms and representations.

Supported types and values for the deserialization:
 - `()` from `undefined` and `null`.
 - `Option` from any value will map `undefined` or `null` to `None` and any other value to `Some(...)`.
 - `bool` from a JavaScript boolean (`false` and `true`).
 - Rust integer (`u8`/`i8`/.../`u128`/`i128`) from a safe JavaScript integer (as matched by [`Number.isSafeInteger`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number/isSafeInteger)).
 - Rust floating number (`f32`/`f64`) from any JavaScript number.
 - `char` from a JavaScript string containing a single codepoint.
 - `String` from any JavaScript string.
 - Rust map (`HashMap`, `BTreeMap`, ...) from any JavaScript iterable producing `[key, value]` pairs (including but not limited to ES2015 `Map`).
 - `HashMap<String, _>` from any plain JavaScript object (`{ key1: value1, ... }`).
 - Rust sequence (tuple, `Vec`, `HashSet`, ...) from any JavaScript iterable (including but not limited to `Array`, ES2015 `Set`, etc.).
 - Rust byte buffer (see [`serde_bytes`](https://github.com/serde-rs/bytes)) from JavaScript `ArrayBuffer` or `Uint8Array`.
 - Typed Rust structure from any plain JavaScript object (`{ key1: value1, ... }`).
 - Rust enum from either a string (`"Variant"`) or a plain object. Specific representation is [controlled](https://serde.rs/enum-representations.html) by `#[serde(...)]` attributes and should be compatible with `serde-json`.

Serialization is compatible with the deserialization, but it's limited to a single representation, so it chooses:
 - `undefined` for `()` or `None`.
 - ES2015 `Map` for Rust maps.
 - `Array` for any Rust sequences.
 - `Uint8Array` for byte buffers.
 - Plain JavaScript object for typed Rust structures.

## License

Licensed under the MIT license. See the [LICENSE](LICENSE) file for details.
