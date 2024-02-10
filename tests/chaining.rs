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
