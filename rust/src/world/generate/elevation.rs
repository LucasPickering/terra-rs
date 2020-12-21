use crate::world::{
    generate::{Generate, TileNoiseFn},
    hex::{HasHexPosition, HexPointMap},
    tile::TileBuilder,
    World, WorldConfig,
};
use derive_more::Display;
use noise::{Fbm, NoiseFn};
use rand::Rng;

/// Generate an elevation map using a noise function.
#[derive(Clone, Debug, Display)]
#[display(fmt = "Elevation Generator")]
pub struct ElevationGenerator;

impl Generate for ElevationGenerator {
    fn generate(
        &self,
        config: &WorldConfig,
        rng: &mut impl Rng,
        tiles: &mut HexPointMap<TileBuilder>,
    ) {
        let noise_fn: TileNoiseFn<Fbm> = TileNoiseFn::new(
            config,
            rng,
            &config.elevation,
            World::ELEVATION_RANGE,
        );
        for tile in tiles.values_mut() {
            tile.set_elevation(noise_fn.get(tile.position()));
        }
    }
}
