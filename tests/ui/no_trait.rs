use bevy_commandify::*;
use bevy::prelude::*;
use bevy::ecs::system::CommandQueue;

#[command(no_trait)]
fn foo(world: &mut World) { }

/// Test that direct trait method fails
fn main() {
    let mut world = World::new();

    let mut queue = CommandQueue::default();
    let mut commands = Commands::new(&mut queue, &mut world);

    commands.foo();

    queue.apply(&mut world);
}