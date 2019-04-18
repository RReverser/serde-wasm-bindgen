use wasm_bindgen::prelude::*;

#[derive(Debug)]
pub struct Error(JsValue);

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        #[wasm_bindgen]
        extern "C" {
            #[wasm_bindgen(js_name = String)]
            pub fn to_string(value: &JsValue) -> String;
        }

        to_string(&self.0).fmt(f)
    }
}

impl std::error::Error for Error {}

impl Error {
    pub fn new<T: std::fmt::Display>(msg: T) -> Self {
        Error(js_sys::Error::new(&msg.to_string()).into())
    }
}

impl serde::ser::Error for Error {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        Error::new(msg)
    }
}

impl serde::de::Error for Error {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        Error::new(msg)
    }
}

// This conversion is needed for `?` to just work when using wasm-bindgen
// imports that return errors from the JS side as Result<T, JsValue>.
impl From<JsValue> for Error {
    fn from(error: JsValue) -> Error {
        Error(error)
    }
}

// This conversion is needed for `?` to just work in wasm-bindgen exports
// that need to return Result<T, JsValue> to throw on the JS side.
impl From<Error> for JsValue {
    fn from(error: Error) -> JsValue {
        error.0
    }
}
