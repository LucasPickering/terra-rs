use crate::{
    util::FloatRange,
    world::{generate::Generate, HexPointMap, Tile, WorldConfig},
};
use noise::{NoiseFn, Perlin, Seedable};

pub struct ElevationMetadata {
    pub elevation: f64,
}

/// Generate an elevation map using a noise function.
#[derive(Copy, Clone, Debug, Default)]
pub struct ElevationGenerator {
    perlin: Perlin,
}

impl ElevationGenerator {
    pub fn new(config: &WorldConfig) -> Self {
        Self {
            perlin: Perlin::new().set_seed(config.seed),
        }
    }
}

impl Generate<(), ElevationMetadata> for ElevationGenerator {
    fn generate(
        &self,
        config: &WorldConfig,
        tiles: HexPointMap<()>,
    ) -> HexPointMap<ElevationMetadata> {
        let normalize =
            |value: isize| -> f64 { value as f64 / config.tile_radius as f64 };

        tiles
            .into_iter()
            .map(|(pos, ())| {
                // This value is in [-1, 1], we want to map it to our
                // elevation range
                let elevation_unit = self.perlin.get([
                    normalize(pos.x),
                    normalize(pos.y),
                    normalize(pos.z),
                ]);
                let elevation = FloatRange::UNIT_RANGE
                    .map_to(&Tile::ELEVATION_RANGE, elevation_unit);

                (pos, ElevationMetadata { elevation })
            })
            .collect()
    }
}
