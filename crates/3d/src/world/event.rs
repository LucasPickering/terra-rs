use terra::WorldConfig;

/// Triggered to *start* a new world generation
pub struct GenerateWorldEvent {
    /// The config to generate the world with
    pub config: WorldConfig,
}
