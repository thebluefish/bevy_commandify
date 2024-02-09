use bevy_commandify::*;
use bevy::prelude::*;
use bevy::ecs::system::CommandQueue;

#[command(no_world)]
fn foo(world: &mut World) { }

/// Test that our generated trait works for Commands, but not World
fn main() {
    let mut world = World::new();

    let mut queue = CommandQueue::default();
    let mut commands = Commands::new(&mut queue, &mut world);

    // success
    commands.foo();
    // failure
    world.foo();

    queue.apply(&mut world);
}