use crate::world::{
    generate::{Generate, TileNoiseFn},
    hex::{HasHexPosition, WorldMap},
    tile::TileBuilder,
    World, WorldConfig,
};
use noise::{Fbm, NoiseFn};
use rand::Rng;

/// Generate an elevation map using a noise function.
#[derive(Debug)]
pub struct ElevationGenerator;

impl Generate for ElevationGenerator {
    fn generate(
        &self,
        config: &WorldConfig,
        rng: &mut impl Rng,
        tiles: &mut WorldMap<TileBuilder>,
    ) {
        let noise_fn: TileNoiseFn<Fbm> =
            TileNoiseFn::new(rng, &config.elevation, World::ELEVATION_RANGE);
        for tile in tiles.iter_mut() {
            tile.set_elevation(noise_fn.get(tile.position()));
        }
    }
}
