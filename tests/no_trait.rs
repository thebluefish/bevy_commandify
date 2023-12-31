use bevy_commandify::*;
use bevy::prelude::*;
use bevy::ecs::system::CommandQueue;

#[command(no_trait)]
fn foo(_world: &mut World) { }

#[test]
fn struct_command_still_works() {
    let mut world = World::new();

    let mut queue = CommandQueue::default();
    let mut commands = Commands::new(&mut queue, &mut world);

    commands.add(FooCommand);

    queue.apply(&mut world);
}