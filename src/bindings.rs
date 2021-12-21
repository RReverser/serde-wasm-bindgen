use js_sys::BigInt;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = BigInt)]
    pub fn from_u64(x: u64) -> BigInt;

    #[wasm_bindgen(js_name = BigInt)]
    pub fn from_i64(x: i64) -> BigInt;

    #[wasm_bindgen(js_name = BigInt)]
    pub fn to_u64(x: BigInt) -> u64;

    #[wasm_bindgen(js_name = BigInt)]
    pub fn to_i64(x: BigInt) -> i64;
}
