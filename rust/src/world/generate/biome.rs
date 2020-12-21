use crate::{
    util::NumRange,
    world::{
        generate::Generate, hex::HexPointMap, tile::TileBuilder, Biome, World,
    },
};
use derive_more::Display;
use rand_pcg::Pcg64;

/// A generator to apply a biome for each tile. The biome is calculated based
/// on elevation and humidity. This won't overwrite any tiles that already have
/// a biome set, so it can be called after other biome-related generators.
#[derive(Copy, Clone, Debug, Default, Display)]
#[display(fmt = "Biome Painter")]
pub struct BiomePainter;

impl Generate for BiomePainter {
    fn generate(&self, tiles: &mut HexPointMap<TileBuilder>, _: &mut Pcg64) {
        // We're going to normalize all the elevations so we can use a
        // consistent set of coefficients below. We don't want to map from the
        // full range though, because 99% of the tiles below sea level we won't
        // be touching (since they're already set to ocean). So map from just
        // above-sea-level elevations. We map end up with a few tiles outside
        // the target range of [0,1], but that's fine because the logic will
        // still give them a biome of some sort.
        let elev_input_range =
            NumRange::new(World::SEA_LEVEL, World::ELEVATION_RANGE.max);

        // Set the biome for each tile, but don't overwrite any existing biomes
        for tile in tiles.values_mut().filter(|tile| tile.biome().is_none()) {
            // Normalize these values so we don't have to update this code when
            // we change the elevation/humidity range bounds
            let elevation =
                elev_input_range.normalize(tile.elevation().unwrap());
            let humidity =
                World::HUMIDITY_RANGE.normalize(tile.humidity().unwrap());

            // A piecewise function to map elevation/humidity to biome.
            // I swear there's logic behind this, I even drew a picture.
            // It looks like a 2d graph with regions sliced out, like a
            // phase diagram.
            // https://en.wikipedia.org/wiki/Phase_diagram#Pressure_vs_temperature
            // Each of these conditions is essentially a 2d function, either
            // x = c or y = mx+b, where elevation is y and humidity is x
            let biome = if elevation >= (-0.1 * humidity + 0.7) {
                Biome::Snow
            } else if humidity <= 0.15 {
                Biome::Desert
            } else if elevation >= (-0.1 * humidity + 0.4) {
                Biome::Alpine
            } else if humidity >= 0.75 {
                Biome::Jungle
            } else if elevation >= (-0.88 * humidity + 0.54) {
                Biome::Forest
            } else {
                Biome::Plains
            };

            tile.set_biome(biome);
        }
    }
}
