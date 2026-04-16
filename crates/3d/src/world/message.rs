use bevy::ecs::message::Message;

/// Triggered to *start* a new world generation
#[derive(Message)]
pub struct GenerateWorldMessage;

/// Triggered to *start* rendering a world, after it has been generated
#[derive(Message)]
pub struct RenderWorldMessage;
