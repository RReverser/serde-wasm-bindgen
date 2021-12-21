use wasm_bindgen::prelude::*;
use js_sys::BigInt;

#[wasm_bindgen]
extern {
    #[wasm_bindgen(js_name = BigInt)]
    pub fn from_u64(x: u64) -> BigInt;

    #[wasm_bindgen(js_name = BigInt)]
    pub fn from_i64(x: i64) -> BigInt;

    #[wasm_bindgen(js_name = BigInt)]
    pub fn to_u64(x: BigInt) -> u64;

    #[wasm_bindgen(js_name = BigInt)]
    pub fn to_i64(x: BigInt) -> i64;
}