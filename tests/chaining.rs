use bevy::ecs::system::CommandQueue;
use bevy::prelude::*;
use bevy_commandify::*;

mod common;
use common::TestUsize;

#[command]
fn foo(world: &mut World, n: usize) -> &mut Self {
    let mut m = world.resource_mut::<TestUsize>();
    **m -= n;
}

#[entity_command]
/// Subtracts `n` from the entity's `TestUsize`
fn bar(world: &mut World, entity: Entity, n: usize) -> &mut Self {
    let mut m = world
        .query::<&mut TestUsize>()
        .get_mut(world, entity)
        .unwrap();
    **m -= n;
}

/// Test that we can chain calls of our commands multiple times in a row
#[test]
fn chain_commands() {
    let mut world = World::new();
    world.insert_resource(TestUsize(30));

    let mut queue = CommandQueue::default();
    let mut commands = Commands::new(&mut queue, &mut world);

    // method call on Commands
    commands.foo(10).foo(10);

    queue.apply(&mut world);

    // method call on World
    world.foo(5).foo(5);

    assert_eq!(**world.resource::<TestUsize>(), 0);
}

/// Test that we can chain calls of our entity_commands multiple times in a row
#[test]
fn chain_entity_commands() {
    let mut world = World::new();
    let entity = world.spawn(TestUsize(30)).id();

    let mut queue = CommandQueue::default();
    let mut commands = Commands::new(&mut queue, &mut world);

    // method call on Commands
    commands.entity(entity).bar(10).bar(10);

    queue.apply(&mut world);

    let mut world_entity = world.entity_mut(entity);

    // method call on EntityWorldMut
    world_entity.bar(5).bar(5);

    assert_eq!(**world.query::<&TestUsize>().single(&world), 0);
}

/// Test that we can spawn an entity, run our command on it, and chain other commands afterwards
#[test]
fn spawn_chain_commands() {
    let mut world = World::new();

    let mut queue = CommandQueue::default();
    let mut commands = Commands::new(&mut queue, &mut world);

    // The operation effectively does nothing since we replace it right after
    commands.spawn(TestUsize(10)).bar(5).insert(TestUsize(100));

    queue.apply(&mut world);

    assert_eq!(**world.query::<&TestUsize>().single(&world), 100);
}

/// Test that we can spawn an entity against the world and chain other commands afterwards
#[test]
fn spawn_chain_world() {
    let mut world = World::new();

    // The operation effectively does nothing since we replace it right after
    world.spawn(TestUsize(10)).bar(5).insert(TestUsize(100));

    assert_eq!(**world.query::<&TestUsize>().single(&world), 100);
}
