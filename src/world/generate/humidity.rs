use crate::world::{
    generate::{Generate, TileBuilder, TileNoiseFn},
    HasHexPosition, HexPointMap, Tile, WorldConfig,
};
use noise::{BasicMulti, NoiseFn};
use std::fmt::{self, Display, Formatter};

/// Generate an humidity map using a noise function.
#[derive(Clone, Debug)]
pub struct HumidityGenerator {
    noise_fn: TileNoiseFn<BasicMulti>,
}

impl HumidityGenerator {
    pub fn new(config: &WorldConfig) -> Self {
        Self {
            noise_fn: TileNoiseFn::new(
                config,
                &config.humidity,
                Tile::HUMDITY_RANGE,
            ),
        }
    }
}

impl Display for HumidityGenerator {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "HumidityGenerator")?;
        Ok(())
    }
}

impl Generate for HumidityGenerator {
    fn generate(&self, tiles: &mut HexPointMap<TileBuilder>) {
        for tile in tiles.values_mut() {
            tile.set_humidity(self.noise_fn.get(tile.position()));
        }
    }
}
