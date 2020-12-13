use crate::world::{
    generate::Generate, hex::HexPointMap, tile::TileBuilder, Biome,
};
use derive_more::Display;
use rand_pcg::Pcg64;

/// A generator to apply a biome for each tile. The biome is calculated based
/// on elevation and humidity.
#[derive(Copy, Clone, Debug, Default, Display)]
#[display(fmt = "Biome Painter")]
pub struct BiomePainter;

/// Calculate the biome for a single tile.
fn calculate_biome(tile: &TileBuilder) -> Biome {
    let elevation = tile.elevation();
    let humidity = tile.humidity();

    // A piecewise function to map elevation/humidity to biome.
    // I swear there's logic behind this, I even drew a picture.
    // It looks like a 2d graph with regions sliced out, like a phase diagram.
    // https://en.wikipedia.org/wiki/Phase_diagram#Pressure_vs_temperature
    if elevation >= (-100.0 * humidity + 700.0) {
        Biome::Snow
    } else if elevation <= 0.15 {
        Biome::Desert
    } else if elevation >= (-100.0 * humidity + 400.0) {
        Biome::Alpine
    } else if humidity >= 0.75 {
        Biome::Jungle
    } else if elevation >= (-880.0 * humidity + 540.0) {
        Biome::Forest
    } else {
        Biome::Plains
    }
}

impl Generate for BiomePainter {
    fn generate(&self, tiles: &mut HexPointMap<TileBuilder>, _: &mut Pcg64) {
        for tile in tiles.values_mut() {
            tile.set_biome(calculate_biome(tile));
        }
    }
}
