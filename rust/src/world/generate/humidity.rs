use crate::world::{
    generate::{Generate, TileNoiseFn},
    hex::{HasHexPosition, HexPointMap},
    tile::TileBuilder,
    World, WorldConfig,
};
use derive_more::Display;
use noise::{BasicMulti, NoiseFn};
use rand::Rng;

/// Generate an humidity map using a noise function.
#[derive(Clone, Debug, Display)]
#[display(fmt = "Humidity Generator")]
pub struct HumidityGenerator;

impl Generate for HumidityGenerator {
    fn generate(
        &self,
        config: &WorldConfig,
        rng: &mut impl Rng,
        tiles: &mut HexPointMap<TileBuilder>,
    ) {
        let noise_fn: TileNoiseFn<BasicMulti> = TileNoiseFn::new(
            config,
            rng,
            &config.humidity,
            World::HUMIDITY_RANGE,
        );
        for tile in tiles.values_mut() {
            tile.set_humidity(noise_fn.get(tile.position()));
        }
    }
}
