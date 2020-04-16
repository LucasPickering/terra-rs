use crate::{
    util::FloatRange,
    world::{
        generate::{elevation::ElevationMetadata, Generate},
        HexPointMap, Tile, WorldConfig,
    },
};
use noise::{NoiseFn, Perlin, Seedable};

pub struct HumidityMetadata {
    pub elevation: f64,
    pub humidity: f64,
}

/// Generate an humidity map using a noise function.
#[derive(Copy, Clone, Debug, Default)]
pub struct HumidityGenerator {
    perlin: Perlin,
}

impl HumidityGenerator {
    pub fn new(config: &WorldConfig) -> Self {
        Self {
            perlin: Perlin::new().set_seed(config.seed),
        }
    }
}

impl Generate<ElevationMetadata, HumidityMetadata> for HumidityGenerator {
    fn generate(
        &self,
        config: &WorldConfig,
        tiles: HexPointMap<ElevationMetadata>,
    ) -> HexPointMap<HumidityMetadata> {
        let normalize =
            |value: isize| -> f64 { value as f64 / config.tile_radius as f64 };

        tiles
            .into_iter()
            .map(|(pos, prev)| {
                // This value is in [-1, 1], we want to map it to our
                // humidity range
                let humidity_unit = self.perlin.get([
                    normalize(pos.x),
                    normalize(pos.y),
                    normalize(pos.z),
                ]);
                let humidity = FloatRange::UNIT_RANGE
                    .map_to(&Tile::HUMDITY_RANGE, humidity_unit);

                (
                    pos,
                    HumidityMetadata {
                        elevation: prev.elevation,
                        humidity,
                    },
                )
            })
            .collect()
    }
}
