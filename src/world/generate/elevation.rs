use crate::world::{
    generate::{Generate, NoiseFnConfig, TileNoiseFn},
    HexPointMap, Tile, WorldConfig,
};
use noise::{BasicMulti, NoiseFn};
use std::fmt::{self, Display, Formatter};

pub struct ElevationMetadata {
    pub elevation: f64,
}

/// Generate an elevation map using a noise function.
#[derive(Clone, Debug)]
pub struct ElevationGenerator {
    noise_fn: TileNoiseFn<BasicMulti>,
}

impl ElevationGenerator {
    pub fn new(config: &WorldConfig) -> Self {
        let noise_fn_config = NoiseFnConfig {
            octaves: 12,
            frequency: 1.0,
            lacunarity: 4.0,
            persistence: 0.5,
        };

        Self {
            noise_fn: TileNoiseFn::new(
                config,
                &noise_fn_config,
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

impl Generate<(), ElevationMetadata> for ElevationGenerator {
    fn generate(
        &self,
        tiles: HexPointMap<()>,
    ) -> HexPointMap<ElevationMetadata> {
        tiles
            .into_iter()
            .map(|(pos, ())| {
                (
                    pos,
                    ElevationMetadata {
                        elevation: self.noise_fn.get(pos),
                    },
                )
            })
            .collect()
    }
}
