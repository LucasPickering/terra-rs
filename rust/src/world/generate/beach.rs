use crate::world::{
    generate::Generate,
    hex::{HasHexPosition, HexPointMap},
    tile::TileBuilder,
    Biome, BiomeType,
};
use derive_more::Display;
use rand_pcg::Pcg64;

/// Any coastal tile at/under this elevation will be beach, anything over will
/// be cliff
const MAX_BEACH_ELEV: f64 = 5.0;

/// Convert coastal tiles (tiles adjacent to ocean) into beach or cliff,
/// depending on their elevation.
#[derive(Copy, Clone, Debug, Default, Display)]
#[display(fmt = "Beach Generator")]
pub struct BeachGenerator;

impl Generate for BeachGenerator {
    fn generate(&self, tiles: &mut HexPointMap<TileBuilder>, _: &mut Pcg64) {
        // Find every tile that's adjacent to ocean/coast, which doesn't already
        // have a biome. Then set each one to either beach or cliff, based on
        // its elevation. We have to do this in a bit of a jank way because we
        // can't call tiles.adjacents() once we've grabbed a mutable reference.
        // So we have to pull the positions of the target tiles into a separate
        // set, then mutate those.
        // Potential optimization? - do this in one pass

        let to_paint: HexPointMap<()> = tiles
            .values()
            .filter(|tile| {
                tile.biome().is_none()
                    && tiles.adjacents(tile.position()).any(|(_, adj_tile)| {
                        match adj_tile.biome() {
                            Some(biome) => {
                                biome.biome_type() == BiomeType::Water
                            }
                            None => false,
                        }
                    })
            })
            .map(|tile| tile.position())
            .collect();

        for tile in tiles.values_mut() {
            if to_paint.contains_key(&tile.position()) {
                let biome = if tile.elevation().unwrap() <= MAX_BEACH_ELEV {
                    Biome::Beach
                } else {
                    Biome::Cliff
                };
                tile.set_biome(biome);
            }
        }
    }
}
