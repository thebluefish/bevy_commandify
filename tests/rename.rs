use bevy::ecs::system::CommandQueue;
use bevy::prelude::*;
use bevy_commandify::*;

mod common;
use common::TestUsize;

#[command(name = "sub")]
fn foo(world: &mut World, n: usize) {
    let mut m = world.resource_mut::<TestUsize>();
    **m -= n;
}

/// The `name` attribute should affect all three ways of calling the command
#[test]
fn foo_becomes_sub() {
    let mut world = World::new();
    world.insert_resource(TestUsize(20));

    let mut queue = CommandQueue::default();
    let mut commands = Commands::new(&mut queue, &mut world);

    // method call on Commands
    commands.sub(10);
    // calling method on trait directly
    CommandsSubExt::sub(&mut commands, 5);
    // adding command struct
    commands.add(SubCommand { n: 5 });

    queue.apply(&mut world);

    assert_eq!(**world.resource::<TestUsize>(), 0);
}
