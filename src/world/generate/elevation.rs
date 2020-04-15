use crate::world::{generate::Generate, HexPointMap, WorldConfig};

pub struct ElevationMetadata {
    elevation: f64,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct ElevationGenerator;

impl Generate<(), ElevationMetadata> for ElevationGenerator {
    fn generate(
        &self,
        _config: &WorldConfig,
        tiles: HexPointMap<()>,
    ) -> HexPointMap<ElevationMetadata> {
        tiles
            .into_iter()
            .map(|(pos, ())| (pos, ElevationMetadata { elevation: 10.0 }))
            .collect()
    }
}
