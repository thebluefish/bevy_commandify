use bevy::ecs::system::CommandQueue;
use bevy::prelude::*;
use bevy_commandify::*;

mod common;
use common::TestUsize;

#[command]
/// Subtracts `n` from the `TestUsize` resource
fn foo(world: &mut World, n: usize) {
    let mut m = world.resource_mut::<TestUsize>();
    **m -= n;
}

#[entity_command]
/// Subtracts `n` from the entity's `TestUsize`
fn bar(world: &mut World, entity: Entity, n: usize) {
    let mut m = world
        .query::<&mut TestUsize>()
        .get_mut(world, entity)
        .unwrap();
    **m -= n;
}

/// all three ways of calling the command
#[test]
fn command() {
    let mut world = World::new();
    world.insert_resource(TestUsize(20));

    let mut queue = CommandQueue::default();
    let mut commands = Commands::new(&mut queue, &mut world);

    // method call on Commands
    commands.foo(10);
    // calling method on trait directly
    CommandsFooExt::foo(&mut commands, 5);
    // adding command struct
    commands.add(FooCommand { n: 5 });

    queue.apply(&mut world);

    assert_eq!(**world.resource::<TestUsize>(), 0);
}

/// all three ways of calling the entity_command
#[test]
fn entity_command() {
    let mut world = World::new();
    let entity = world.spawn(TestUsize(20)).id();

    let mut queue = CommandQueue::default();
    let mut commands = Commands::new(&mut queue, &mut world);

    // method call on Commands
    commands.entity(entity).bar(10);
    // calling method on trait directly
    EntityCommandsBarExt::bar(&mut commands.entity(entity), 5);
    // adding command struct
    commands.entity(entity).add(BarEntityCommand { n: 5 });

    queue.apply(&mut world);

    assert_eq!(**world.query::<&TestUsize>().single(&world), 0);
}
