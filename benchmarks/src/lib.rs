use serde::{de::DeserializeOwned, Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;

mod color;
mod prim_str;

#[wasm_bindgen(start)]
pub fn init_console() {
    console_error_panic_hook::set_once();
}

macro_rules! serde_impl {
    (|$input:ident| {
        $($feature:literal => $(# $attr:tt)* {
            parse: $parse:expr,
            serialize: $serialize:expr,
        },)*
    }) => {
        $(
            #[cfg(feature = $feature)]
            $(# $attr)*
            mod serde_impl {
                use super::*;

                pub fn parse<T: DeserializeOwned>($input: JsValue) -> T {
                    $parse
                }

                pub fn serialize<T: Serialize>($input: &T) -> JsValue {
                    $serialize
                }
            }
        )*
    };
}

serde_impl!(|input| {
    "serde-wasm-bindgen" => {
        parse: serde_wasm_bindgen::from_value(input).unwrap_throw(),
        serialize: {
            const SERIALIZER: serde_wasm_bindgen::Serializer = serde_wasm_bindgen::Serializer::json_compatible();
            input.serialize(&SERIALIZER).unwrap_throw()
        },
    },

    "serde-json" => #[allow(deprecated)] {
        parse: input.into_serde().unwrap_throw(),
        serialize: JsValue::from_serde(input).unwrap_throw(),
    },

    "msgpack" => {
        parse: {
            #[wasm_bindgen(module = "@msgpack/msgpack")]
            extern "C" {
                fn encode(input: &JsValue) -> Vec<u8>;
            }

            let input = encode(&input);
            rmp_serde::from_slice(&input).unwrap_throw()
        },

        serialize: {
            #[wasm_bindgen(module = "@msgpack/msgpack")]
            extern "C" {
                fn decode(input: &[u8]) -> JsValue;
            }

            let input = rmp_serde::to_vec(input).unwrap_throw();
            decode(&input)
        },
    },
});

macro_rules! datasets {
    ($($mod:ident :: $ty:ident,)*) => {
        $(
            mod $mod;

            #[wasm_bindgen]
            #[derive(Serialize, Deserialize)]
            pub struct $ty($mod::$ty);

            #[wasm_bindgen]
            impl $ty {
                #[wasm_bindgen]
                pub fn parse(input: JsValue) -> $ty {
                    serde_impl::parse(input)
                }

                #[wasm_bindgen]
                pub fn serialize(&self) -> JsValue {
                    serde_impl::serialize(self)
                }
            }
        )*
    };
}

datasets! {
    canada::Canada,
    citm_catalog::CitmCatalog,
    twitter::Twitter,
}
