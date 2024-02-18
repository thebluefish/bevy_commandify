use bevy::ecs::system::CommandQueue;
use bevy::prelude::*;
use bevy_commandify::*;

mod common;
use common::TestUsize;

#[command]
fn foo(world: &mut World, mut n: usize) {
    n *= 2;
    let mut m = world.resource_mut::<TestUsize>();
    **m -= n;
}

#[entity_command]
/// Subtracts `n` from the entity's `TestUsize`
fn bar(world: &mut World, entity: Entity, mut n: usize) {
    n *= 2;
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
    world.insert_resource(TestUsize(50));

    let mut queue = CommandQueue::default();
    let mut commands = Commands::new(&mut queue, &mut world);

    // method call on Commands
    commands.foo(5);
    // calling method on trait directly
    CommandsFooExt::foo(&mut commands, 5);
    // adding command struct
    commands.add(FooCommand { n: 5 });

    queue.apply(&mut world);

    // method call on World
    world.foo(5);
    // calling method on trait directly
    CommandsFooExt::foo(&mut world, 5);

    assert_eq!(**world.resource::<TestUsize>(), 0);
}

/// all three ways of calling the entity_command
#[test]
fn entity_command() {
    let mut world = World::new();
    let entity = world.spawn(TestUsize(50)).id();

    let mut queue = CommandQueue::default();
    let mut commands = Commands::new(&mut queue, &mut world);

    // method call on Commands
    commands.entity(entity).bar(5);
    // calling method on trait directly
    EntityCommandsBarExt::bar(&mut commands.entity(entity), 5);
    // adding command struct
    commands.entity(entity).add(BarEntityCommand { n: 5 });

    queue.apply(&mut world);

    let mut world_entity = world.entity_mut(entity);

    // method call on EntityWorldMut
    world_entity.bar(5);
    // calling method on trait directly
    EntityCommandsBarExt::bar(&mut world_entity, 5);

    assert_eq!(**world.query::<&TestUsize>().single(&world), 0);
}
