use crate::world::{
    generate::{elevation::ElevationMetadata, Generate, TileNoiseFn},
    HexPointMap, Tile, WorldConfig,
};
use noise::{BasicMulti, NoiseFn};
use std::fmt::{self, Display, Formatter};

pub struct HumidityMetadata {
    pub elevation: f64,
    pub humidity: f64,
}

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

impl Generate<ElevationMetadata, HumidityMetadata> for HumidityGenerator {
    fn generate(
        &self,
        tiles: HexPointMap<ElevationMetadata>,
    ) -> HexPointMap<HumidityMetadata> {
        tiles
            .into_iter()
            .map(|(pos, prev)| {
                (
                    pos,
                    HumidityMetadata {
                        elevation: prev.elevation,
                        humidity: self.noise_fn.get(pos),
                    },
                )
            })
            .collect()
    }
}
