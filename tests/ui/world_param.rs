use bevy_commandify::*;

#[command]
fn foo(n_: usize) { }

/// Test that a command requires a `&mut World` fn parameter
fn main() { }