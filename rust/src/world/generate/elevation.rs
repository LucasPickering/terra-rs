use crate::world::{
    generate::{Generate, TileNoiseFn},
    hex::{HasHexPosition, HexPointMap},
    tile::TileBuilder,
    World, WorldConfig,
};
use derive_more::Display;
use noise::{Fbm, NoiseFn};
use rand_pcg::Pcg64;

/// Generate an elevation map using a noise function.
#[derive(Clone, Debug, Display)]
#[display(fmt = "Elevation Generator")]
pub struct ElevationGenerator {
    noise_fn: TileNoiseFn<Fbm>,
}

impl ElevationGenerator {
    pub fn new(config: &WorldConfig) -> Self {
        Self {
            noise_fn: TileNoiseFn::new(
                config,
                &config.elevation,
                World::ELEVATION_RANGE,
            ),
        }
    }
}

impl Generate for ElevationGenerator {
    fn generate(&self, tiles: &mut HexPointMap<TileBuilder>, _: &mut Pcg64) {
        for tile in tiles.values_mut() {
            tile.set_elevation(self.noise_fn.get(tile.position()));
        }
    }
}
