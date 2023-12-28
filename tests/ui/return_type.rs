use bevy_commandify::*;

#[command(fail)]
fn foo(world: &mut World) -> usize { }

fn main() { }