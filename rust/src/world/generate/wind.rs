use crate::world::{
    generate::{Generate, WorldBuilder},
    hex::{HexAxialDirection, HexAxis},
};
use rand::{prelude::IteratorRandom, Rng};
use strum::IntoEnumIterator;

/// Generate a prevailing wind direction for the world. In the future we could
/// extend this to have local wind patterns based on terrain, but not right now.
#[derive(Debug)]
pub struct WindGenerator;

impl Generate for WindGenerator {
    fn generate(&self, world: &mut WorldBuilder) {
        let axis = HexAxis::iter().choose_stable(&mut world.rng).unwrap();
        let positive: bool = world.rng.gen_bool(0.5);
        world.wind_direction = Some(HexAxialDirection { axis, positive });
    }
}
