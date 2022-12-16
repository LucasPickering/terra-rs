pub mod range;
pub mod unit;

use std::cmp::Ordering;

/// A macro to unwrap an option to its `Some` value, and panic if `None`. This
/// is the same as [Option::unwrap], except that it accepts a format string
/// and format arguments, allowing for more flexibility in error messages.
#[macro_export]
macro_rules! unwrap {
    ($opt:expr, $fmt:expr, $($arg:tt)*) => {
        match $opt {
            Some(v) => v,
            None => panic!($fmt, $($arg)*),
        }
    };
}

/// A macro to measure the evaluation time of an expression. Wraps an
/// expression, and outputs a tuple of the value of the expression with the
/// elapsed time, as a [Duration](std::time::Duration).
#[cfg(not(target_arch = "wasm32"))]
#[macro_export]
macro_rules! timed {
    ($label:expr, $ex:expr) => {
        timed!($label, log::Level::Debug, $ex)
    };
    ($label:expr, $log_level:expr, $ex:expr) => {{
        let now = std::time::Instant::now();
        let value = $ex;
        let elapsed = now.elapsed();
        log::log!($log_level, "{} took {} ms", $label, elapsed.as_millis());
        value
    }};
}

/// Re-implementation of the above macro for wasm
#[cfg(target_arch = "wasm32")]
#[doc(hidden)]
#[macro_export]
macro_rules! timed {
    ($label:expr, $ex:expr) => {
        // log level does nothing on the wasm version
        timed!($label, log::Level::Debug, $ex)
    };
    ($label:expr, $log_level:expr, $ex:expr) => {{
        use web_sys::console;

        // https://developer.mozilla.org/en-US/docs/Web/API/console/time
        console::time_with_label($label);
        let value = $ex;
        console::time_end_with_label($label);
        value
    }};
}

/// Compare two `PartialOrd` values dangerously. If the partial comparison
/// fails (returns `None`), this will panic. This is useful if you have floats
/// that you know for a fact will not be `NaN`.
pub fn cmp_unwrap<T: PartialOrd>(a: &T, b: &T) -> Ordering {
    a.partial_cmp(b).unwrap()
}

/// Round a value to the nearest multiple of a given arbitrary interval.
/// This will panic for any non-positive interval. Supports rounding for any
/// unit that can be converted to/from an `f64`.
pub fn round<T: From<f64> + Into<f64>>(value: T, interval: T) -> T {
    let value: f64 = value.into();
    let interval: f64 = interval.into();
    assert!(interval > 0.0, "Rounding interval must be positive");
    let rounded = (value / interval).round() * interval;
    rounded.into()
}

/// Calculate the length of a world (the number of tiles it contains) based on
/// its radius. Radius 0 means 1 tile, 1 is 7 tiles, 2 is 19, etc.
pub fn world_len(radius: u16) -> usize {
    // We'll always have 3r^2+3r+1 tiles (a reduction of a geometric sum).
    // f(0) = 1, and we add 6r tiles for every step after that, so:
    // 1, (+6) 7, (+12) 19, (+18) 37, ...
    let r = radius as usize;
    3 * r * r + 3 * r + 1
}

// Serialize a TilePointMap as a list instead of a map. This is useful because
// TilePoints generally shouldn't be used as serialized map keys, since JSON and
// other formats don't support complex keys.
pub mod serde_tile_point_map_to_vec {
    use crate::{HasHexPosition, TilePoint, TilePointMap};
    use serde::{
        ser::SerializeSeq, Deserialize, Deserializer, Serialize, Serializer,
    };

    /// Serialize a tile point map as a list
    pub fn serialize<T, S>(
        map: &TilePointMap<T>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        T: Serialize,
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(map.len()))?;
        for tile in map.values() {
            seq.serialize_element(tile)?;
        }
        seq.end()
    }

    /// Deserialize a list of values into a map. The deserialized type must
    /// implement [HasHexPosition] so that we can derive a [TilePoint] for each
    /// element to use as its map key.
    pub fn deserialize<'de, T, D>(
        deserializer: D,
    ) -> Result<TilePointMap<T>, D::Error>
    where
        T: Deserialize<'de> + HasHexPosition<Point = TilePoint>,
        D: Deserializer<'de>,
    {
        let vec: Vec<T> = Vec::deserialize(deserializer)?;
        Ok(vec
            .into_iter()
            .map(|element| (element.position(), element))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn test_round_to() {
        assert_approx_eq!(round(3.0, 1.0), 3.0);
        assert_approx_eq!(round(3.4, 1.0), 3.0);
        assert_approx_eq!(round(3.5, 1.0), 4.0);
        assert_approx_eq!(round(-3.4, 1.0), -3.0);
        assert_approx_eq!(round(-3.5, 1.0), -4.0);

        assert_approx_eq!(round(3.5, 10.0), 0.0);
        assert_approx_eq!(round(6.8, 10.0), 10.0);
        assert_approx_eq!(round(65.0, 10.0), 70.0);

        assert_approx_eq!(round(65.0, 0.7), 65.1);
        assert_approx_eq!(round(-65.0, 0.7), -65.1);

        assert_approx_eq!(round(6.5, 1.5), 6.0);
        assert_approx_eq!(round(-6.5, 1.5), -6.0);
    }

    #[test]
    fn test_world_len() {
        assert_eq!(world_len(0), 1);
        assert_eq!(world_len(1), 7);
        assert_eq!(world_len(2), 19);
        assert_eq!(world_len(3), 37);
    }
}
