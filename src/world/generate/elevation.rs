use crate::world::{
    generate::{Generate, TileBuilder, TileNoiseFn},
    HasHexPosition, HexPointMap, Tile, WorldConfig,
};
use noise::{BasicMulti, NoiseFn};
use std::fmt::{self, Display, Formatter};

/// Generate an elevation map using a noise function.
#[derive(Clone, Debug)]
pub struct ElevationGenerator {
    noise_fn: TileNoiseFn<BasicMulti>,
}

impl ElevationGenerator {
    pub fn new(config: &WorldConfig) -> Self {
        Self {
            noise_fn: TileNoiseFn::new(
                config,
                &config.elevation,
                Tile::ELEVATION_RANGE,
            ),
        }
    }
}

impl Display for ElevationGenerator {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "ElevationGenerator")?;
        Ok(())
    }
}

impl Generate for ElevationGenerator {
    fn generate(&self, tiles: &mut HexPointMap<TileBuilder>) {
        for tile in tiles.values_mut() {
            tile.set_elevation(self.noise_fn.get(tile.position()));
        }
    }
}
