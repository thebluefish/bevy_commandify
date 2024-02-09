// prevent this test from seeing the bevy crate, forcing it to use `bevy_ecs` as the crate root for our macro
extern crate self as bevy;

use bevy_commandify::*;
use bevy_ecs::prelude::*;
use bevy_ecs::system::CommandQueue;

#[command(ecs = bevy_ecs)]
fn foo(_world: &mut World) {}

#[entity_command(bevy_ecs)]
fn bar(_world: &mut World, _entity: Entity) {}

/// The `ecs` attribute should point this macro to the correct `bevy_ecs`-equivalent root
#[test]
fn ecs_name() {
    let mut world = World::new();

    let mut queue = CommandQueue::default();
    let mut commands = Commands::new(&mut queue, &mut world);

    commands.foo();
    commands.spawn_empty().bar();

    queue.apply(&mut world);
}
