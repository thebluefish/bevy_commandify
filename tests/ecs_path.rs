extern crate self as bevy;

use bevy_commandify::*;
use bevy_ecs::prelude::*;
use bevy_ecs::system::CommandQueue;

#[command(ecs = bevy_ecs)]
fn foo(_world: &mut World) {}

#[command(bevy_ecs)]
fn bar(_world: &mut World) {}

/// The `ecs` attribute should point this macro to the correct `bevy_ecs`-equivalent root
#[test]
fn ecs_name() {
    let mut world = World::new();

    let mut queue = CommandQueue::default();
    let mut commands = Commands::new(&mut queue, &mut world);

    commands.foo();

    queue.apply(&mut world);
}
