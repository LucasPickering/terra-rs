use crate::{
    util::FloatRange,
    world::{
        generate::{elevation::ElevationMetadata, Generate},
        HexPoint, HexPointMap, Tile, WorldConfig,
    },
};
use noise::{BasicMulti, MultiFractal, NoiseFn, Seedable};
use std::fmt::{self, Display, Formatter};

pub struct HumidityMetadata {
    pub elevation: f64,
    pub humidity: f64,
}

/// Generate an humidity map using a noise function.
#[derive(Clone, Debug)]
pub struct HumidityGenerator {
    tile_pos_range: FloatRange,
    noise_fn: BasicMulti,
}

impl HumidityGenerator {
    pub fn new(config: &WorldConfig) -> Self {
        let noise_fn = BasicMulti::new()
            .set_seed(config.seed)
            .set_frequency(8.0)
            .set_lacunarity(2.0)
            .set_persistence(0.5)
            .set_octaves(6);

        let radius_f = config.tile_radius as f64;
        Self {
            tile_pos_range: FloatRange::new(-radius_f, radius_f),
            noise_fn,
        }
    }

    fn generate_humidity(&self, pos: HexPoint) -> f64 {
        // This value is in [-1, 1], we want to map it to the noise function's
        // domain
        let normalize = |v: isize| {
            self.tile_pos_range
                .map_to(&FloatRange::UNIT_RANGE, v as f64)
        };
        let normalized_pos =
            [normalize(pos.x), normalize(pos.y), normalize(pos.z)];

        // Generate noise from the position
        let humidity_unit = self.noise_fn.get(normalized_pos);

        // And map from the noise func's range to humidity
        FloatRange::UNIT_RANGE.map_to(&Tile::HUMDITY_RANGE, humidity_unit)
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
                        humidity: self.generate_humidity(pos),
                    },
                )
            })
            .collect()
    }
}
