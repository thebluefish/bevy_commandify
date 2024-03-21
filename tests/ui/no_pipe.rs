use bevy_commandify::*;

#[command(pipe=)]
fn foo(world: &mut World) { }

/// Test that a command with a pipe specifies the pipe fn.
fn main() { }