use serde::{Deserialize, Serialize};
use std::fmt::Display;
use validator::Validate;
use wasm_bindgen::{prelude::*, JsCast};

pub fn to_js_error(error: impl Display) -> JsValue {
    js_sys::Error::new(&error.to_string()).into()
}

/// An extension trait for `Result` to allow us to add custom methods
pub trait ResultExt<T, E> {
    /// Helper to convert any result to a result with a JS error value.
    fn into_js(self) -> Result<T, JsValue>;
}

impl<T, E: Display> ResultExt<T, E> for Result<T, E> {
    fn into_js(self) -> Result<T, JsValue> {
        self.map_err(to_js_error)
    }
}

/// Verify that the given JS object is a valid config. Return the validated
/// config, with all defaults populated, if it's valid. The return value will be
/// **re-serialized** into a strictly typed object. So it will be a JS object,
/// not a Rust value, BUT it will have TS type safety. Return an error if input
/// isn't valid.
///
/// Because validation populates missing fields, you can "validate" an empty
/// object to get the default config.
pub fn validate_config<R, O>(input: JsValue) -> Result<O, JsValue>
where
    R: Serialize + for<'a> Deserialize<'a> + Validate,
    O: JsCast,
{
    // Deserialize the config then validate it manually
    let config: R = JsValue::into_serde(&input).into_js()?;
    config.validate().into_js()?;

    // Re-serialize it back into a JS object. This assumes that
    // the TS interface type correctly matches the serialization
    // format for the Rust config type. Not great, but it's the
    // best option.
    Ok(JsValue::from_serde(&config).unwrap().unchecked_into())
}
