use bevy::prelude::{Commands, Component, Entity};
use terra::{HasHexPosition, Tile, TilePointMap};

/// A singleton component to store a mapping of tile position to entity ID. This
/// makes it possible to quickly look up tiles by their position. This should
/// be initialized during world generation, when all tiles are spawned. Tiles
/// should be spawned using [Self::spawn_tile], to ensure they get added to the
/// map as well as the ECS.
#[derive(Component, Debug, Default)]
pub struct TileStorage {
    tile_entities: TilePointMap<Entity>,
}

impl TileStorage {
    /// Spawn a tile into the ECS, and store its entity ID for later. This is
    /// how all tiles should be spawned.
    pub fn spawn_tile(&mut self, commands: &mut Commands, tile: Tile) {
        let position = tile.position();
        let entity = commands.spawn(tile).id();
        self.tile_entities.insert(position, entity);
    }
}
