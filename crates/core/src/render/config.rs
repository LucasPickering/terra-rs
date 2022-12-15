use crate::{Meter3, TileLens};
use serde::{Deserialize, Serialize};
use validator::Validate;

/// Configuration specific to visually rendering a world. These options have
/// absolutely no bearing on world _generation_, only on the visual
/// presentation. In other words, if you generate a world then output to a
/// non-visual format (e.g. JSON or binary), these options will **never**
/// affect that output.
///
/// Not all render options apply to all render output formats, e.g.
/// `vertical_scale` is irrelevant for 2D rendering like SVG. The documentation
/// for each field will list which formats it applies to.
#[derive(Copy, Clone, Debug, Serialize, Deserialize, Validate)]
#[cfg_attr(feature = "bevy", derive(bevy_ecs::system::Resource))]
#[serde(default)]
pub struct RenderConfig {
    /// The vertical scale factor applied to each tile. This impacts the
    /// _presentation_ of each tile's elevation, but **does not factor into**
    /// the tile's underlying elevation value. With a scale of 1.0, one meter
    /// of elevation will be the same distance as the length of one side of a
    /// tile's top or bottom face, i.e. one sixth of the tile's perimeter.
    ///
    /// TODO validate claim here about scale equivalency
    ///
    /// ## Relevant Formats
    /// - STL
    #[validate(range(min = 0.001))]
    pub vertical_scale: f64,

    /// A tile lens controls what information is used to determine the
    /// appearance of each tile. For example, [TileLens::Biome] means color
    /// is based entirely on biome.
    ///
    /// ## Relevant Formats
    /// - SVG
    pub tile_lens: TileLens,

    /// Should geographic features (lakes, rivers, etc.) be visible? See
    /// [crate::GeoFeature] for a full list
    ///
    /// ## Relevant Formats
    /// - SVG
    pub show_features: bool,

    /// A **soft** cap on how runoff values are rendered. Any runoff at or
    /// above this bound will be rendered as the "max" visual value, which
    /// could be color, lake size, etc.
    ///
    /// ## Relevant Formats
    /// - SVG
    pub max_runoff: Meter3,

    /// A **soft** cap on how runoff flow totals (runoff ingress and egress)
    /// are rendered. Any runoff flow value at or above this bound will be
    /// rendered as the "max" visual value, which could be color, river width,
    /// etc.
    ///
    /// ## Relevant Formats
    /// - SVG
    pub max_runoff_flow: Meter3,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            vertical_scale: 1.0,
            tile_lens: TileLens::Surface,
            show_features: true,
            max_runoff: Meter3(5.0),
            max_runoff_flow: Meter3(1000.0),
        }
    }
}
