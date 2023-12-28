use bevy_commandify::*;

#[command(name = foo::bar)]
fn foo(world: &mut World) { }

/// Test that a multi-part path fails
fn main() { }