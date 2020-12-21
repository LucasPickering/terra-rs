use crate::world::{
    generate::{Generate, TileNoiseFn},
    hex::{HasHexPosition, HexPointMap},
    tile::TileBuilder,
    World, WorldConfig,
};
use derive_more::Display;
use noise::{BasicMulti, NoiseFn};
use rand_pcg::Pcg64;

/// Generate an humidity map using a noise function.
#[derive(Clone, Debug, Display)]
#[display(fmt = "Humidity Generator")]
pub struct HumidityGenerator {
    noise_fn: TileNoiseFn<BasicMulti>,
}

impl HumidityGenerator {
    pub fn new(config: &WorldConfig) -> Self {
        Self {
            noise_fn: TileNoiseFn::new(
                config,
                &config.humidity,
                World::HUMIDITY_RANGE,
            ),
        }
    }
}

impl Generate for HumidityGenerator {
    fn generate(&self, tiles: &mut HexPointMap<TileBuilder>, _: &mut Pcg64) {
        for tile in tiles.values_mut() {
            tile.set_humidity(self.noise_fn.get(tile.position()));
        }
    }
}
