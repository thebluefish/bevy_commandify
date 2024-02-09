use bevy_commandify::*;

#[entity_command]
fn foo(world: &mut World) { }

/// Test that an entity_command requires an `Entity` fn parameter
fn main() { }