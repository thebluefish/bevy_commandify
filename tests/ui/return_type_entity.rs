use bevy_commandify::*;

#[entity_command]
fn foo(world: &mut World) -> Command { }

fn main() { }