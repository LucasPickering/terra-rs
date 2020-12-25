use crate::world::{
    generate::{Generate, TileNoiseFn},
    hex::{HasHexPosition, WorldMap},
    tile::TileBuilder,
    World, WorldConfig,
};
use noise::{Fbm, NoiseFn};
use rand::Rng;

/// Generate an humidity map using a noise function.
#[derive(Debug)]
pub struct HumidityGenerator;

impl Generate for HumidityGenerator {
    fn generate(
        &self,
        config: &WorldConfig,
        rng: &mut impl Rng,
        tiles: &mut WorldMap<TileBuilder>,
    ) {
        let noise_fn: TileNoiseFn<Fbm> =
            TileNoiseFn::new(rng, &config.humidity, World::HUMIDITY_RANGE);
        for tile in tiles.iter_mut() {
            tile.set_humidity(noise_fn.get(tile.position()));
        }
    }
}
