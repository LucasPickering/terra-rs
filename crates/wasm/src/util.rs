use serde::{Deserialize, Serialize};
use std::{marker::PhantomData, ops::Deref};
use terra::{anyhow, validator::Validate, RenderConfig, WorldConfig};
use wasm_bindgen::{prelude::*, JsCast};

use crate::{RenderConfigObject, WorldConfigObject};

/// An extension trait for `Result` to allow us to add custom methods
pub trait ResultExt<T, E> {
    /// Helper to convert any result to a result with a JS error value.
    fn into_js(self) -> Result<T, JsValue>;
}

impl<T> ResultExt<T, anyhow::Error> for Result<T, anyhow::Error> {
    fn into_js(self) -> Result<T, JsValue> {
        self.map_err(|error| {
            format!("{:?}\n{}", error, error.backtrace()).into()
        })
    }
}

/// A little container for consolidating functionality related to mapping
/// config objects between JS values and Rust values.
pub struct ConfigHelper<T, J>
where
    T: Default + Serialize + for<'a> Deserialize<'a> + Validate,
    J: Deref<Target = JsValue> + JsCast,
{
    phantom_t: PhantomData<T>,
    phantom_j: PhantomData<J>,
}

impl<T, J> ConfigHelper<T, J>
where
    T: Default + Serialize + for<'a> Deserialize<'a> + Validate,
    J: Deref<Target = JsValue> + JsCast,
{
    pub fn new() -> Self {
        Self {
            phantom_t: PhantomData,
            phantom_j: PhantomData,
        }
    }

    /// Get the default world config as a JS object.
    pub fn default(&self) -> J {
        JsValue::from_serde(&T::default()).unwrap().unchecked_into()
    }

    /// Deserialize a JS object into a [WorldConfig]. The input should be an
    /// **object**, not a JSON string. Will return an error if deserialization
    /// fails in any way.
    pub fn deserialize(&self, input: J) -> Result<T, JsValue> {
        JsValue::into_serde(&input).map_err(|err| {
            format!("Error deserializing value: {:?}", err).into()
        })
    }

    /// Verify that the given JS object is a valid Terra world config. Return
    /// the validated config, with all defaults populated, if it's valid. Return
    /// an error if it isn't.
    pub fn validate(&self, input: J) -> Result<J, JsValue> {
        // Deserialize the config then validate it manually
        let config = self.deserialize(input)?;
        config.validate().map_err::<JsValue, _>(|err| {
            format!("Invalid value: {:?}", err).into()
        })?;
        // Re-serialize it back into a JS object
        Ok(JsValue::from_serde(&config).unwrap().unchecked_into())
    }
}

pub type WorldConfigHelper = ConfigHelper<WorldConfig, WorldConfigObject>;
pub type RenderConfigHelper = ConfigHelper<RenderConfig, RenderConfigObject>;
