use crate::world::{
    generate::{Generate, TileNoiseFn, WorldBuilder},
    hex::HasHexPosition,
    World,
};
use noise::Fbm;

/// Generate an humidity map using a noise function.
#[derive(Debug)]
pub struct HumidityGenerator;

impl Generate for HumidityGenerator {
    fn generate(&self, world: &mut WorldBuilder) {
        let noise_fn: TileNoiseFn<Fbm> = TileNoiseFn::new(
            &mut world.rng,
            &world.config.humidity,
            World::HUMIDITY_RANGE,
        );
        for tile in world.tiles.iter_mut() {
            tile.set_humidity(noise_fn.get(tile.position()));
        }
    }
}
