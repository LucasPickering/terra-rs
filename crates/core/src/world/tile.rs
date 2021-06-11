use crate::{
    world::hex::TileDirectionValues, Biome, GeoFeature, HasHexPosition, Meter,
    Meter2, Meter3, TilePoint, World,
};
use serde::{Deserialize, Serialize};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

/// A world is comprised of tiles. Each tile is a hexagon (in 2D renderings) or
/// a hexagonal prism (in 3D renderings). In the case of the prism, a tile's
/// height is determined by its elevation. Tiles **cannot** be stacked.
///
/// A tile has certain geographic properties, and when we combine a bunch of
/// tiles together, we get terrain.
///
/// Tiles can't be constructed directly, they can only be made by the world
/// generation process. See [World::generate]. They also can't be modified after
/// world generation.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tile {
    // These fields are all pub(super) so they can be accessed by the builder
    /// The location of this tile in the world. See [TilePoint] for a
    /// description of the coordinate system. Every tile in the world has a
    /// unique position.
    pub(super) position: TilePoint,

    /// The elevation of this tile, relative to sea level.
    pub(super) elevation: Meter,

    /// Amount of rain that fell on this tile during rain simulation.
    pub(super) rainfall: Meter3,

    /// Amount of runoff water that remains on the tile after runoff
    /// simulation.
    pub(super) runoff: Meter3,

    /// The net amount of runoff gained/lost for this tile in each direction.
    /// Positive values indicate ingress (i.e. runoff came in from that
    /// direction) and negative values indicate egress (i.e. runoff left in
    /// that direction). The value should be positive if the neighbor in that
    /// direction is a higher elevation, and negative if it is lower.
    pub(super) runoff_traversed: TileDirectionValues<Meter3>,

    /// The biome for this tile. Every tile exists in a single biome, which
    /// describes its climate characteristics. See [Biome] for more info.
    pub(super) biome: Biome,

    /// All geographic features on this tile. A geographic feature describes
    /// some unique formation that can appear on a tile. No two features in
    /// this vec can be identical.
    pub(super) features: Vec<GeoFeature>,
}

impl Tile {
    /// The top surface area of a single tile, in abstract units! Note that this
    /// math doesn't line up with [WorldRenderer::TILE_VERTEX_RADIUS] or the
    /// other rendering constants, i.e. if you were to calculate the area of
    /// a hexagon with a radius of `WorldRenderer::TILE_VERTEX_RADIUS`, you
    /// wouldn't get the same value as this constant! This area value only
    /// applies to world space, and it makes certain calculations (e.g. runoff)
    /// simpler.
    ///
    /// TODO maybe we _should_ make this consistent? would it really make
    /// runoff stuff harder?
    pub const AREA: Meter2 = Meter2(1.0);

    /// Get a list of geographic features that appear on this tile. See
    /// [GeoFeature] for more info.
    ///
    /// **Note**: NOT available to WebAssembly. `wasm-bindgen` doesn't support
    /// complex enums, so we can't pass [GeoFeature] across the Wasm boundary.
    pub fn features(&self) -> &[GeoFeature] {
        self.features.as_slice()
    }

    /// Return the elevation of the top of this tile, relative to sea level.
    /// This value is guaranteed to be in the range [Self::ELEVATION_RANGE].
    pub fn elevation(&self) -> Meter {
        self.elevation
    }

    /// Total amount of water that fell on this tile during rain simulation.
    /// This value is guaranteed to be non-negative, but has no hard maximum.
    /// If you need to map a rainfall value to some bounded range, you can use
    /// [Self::RAINFALL_SOFT_RANGE] for a soft maximum.
    pub fn rainfall(&self) -> Meter3 {
        self.rainfall
    }

    /// A normalized (meaning [0,1]) proxy for rainfall. Since rainfall is an
    /// unbounded range, we define an arbitrary soft maximum for it, and
    /// anything at/above that max will map to 1.0 humidity. Anything between
    /// the min (0) and the soft max will map proportionally to [0,1] to
    /// determine humidity.
    ///
    /// This function will **always** return a value in [0,1].
    pub fn humidity(&self) -> f64 {
        World::RAINFALL_SOFT_RANGE
            .value(self.rainfall)
            .clamp()
            .convert::<f64>()
            .normalize()
            .inner()
    }

    /// The amount of water runoff that collected on this tile. This is the
    /// amount of runoff **currently** on the tile after runoff simulation,
    /// **not** the amount of total runoff that passed over the tile.
    pub fn runoff(&self) -> Meter3 {
        self.runoff
    }

    /// Get the total amount of runoff that _entered_ this tile. This is the
    /// **gross** ingress, not the **net**.
    pub fn runoff_ingress(&self) -> Meter3 {
        // Ingress values are positive, so filter out negative values
        std::array::IntoIter::new(self.runoff_traversed.as_array())
            .filter(|v| *v > Meter3(0.0))
            .sum()
    }

    /// Get the total amount of runoff that _exited_ this tile. This is the
    /// **gross** egress, not the **net**.
    pub fn runoff_egress(&self) -> Meter3 {
        // Egress values are negative, so filter out positive values, then
        // negate the sum
        -std::array::IntoIter::new(self.runoff_traversed.as_array())
            .filter(|v| *v < Meter3(0.0))
            .sum::<Meter3>()
    }

    /// Get the tile's biome. Every tile will have exactly on biome assigned.
    /// See [Biome] for more info.
    pub fn biome(&self) -> Biome {
        self.biome
    }
}

impl HasHexPosition for Tile {
    type Point = TilePoint;

    fn position(&self) -> TilePoint {
        self.position
    }
}
