use bevy_commandify::*;

#[command(ok=)]
fn foo(world: &mut World) { }

/// Test that a command with a handler specifies the handler.
fn main() { }