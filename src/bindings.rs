use js_sys::BigInt;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = BigInt)]
    pub fn bigint_to_u64(x: &BigInt) -> u64;

    #[wasm_bindgen(js_name = BigInt)]
    pub fn bigint_to_i64(x: &BigInt) -> i64;

    #[wasm_bindgen(js_name = BigInt)]
    pub fn bigint_from_u64(x: u64) -> BigInt;

    #[wasm_bindgen(js_name = BigInt)]
    pub fn bigint_from_i64(x: i64) -> BigInt;
}
