#![allow(unused_variables)]

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

#[entity_command(name = "bus")]
fn bar(world: &mut World, entity: Entity, n: usize) {
    let mut m = world
        .query::<&mut TestUsize>()
        .get_mut(world, entity)
        .unwrap();
    **m -= n;
}

/// The `name` attribute should affect all three ways of calling the command
#[test]
fn foo_becomes_sub() {
    let mut world = World::new();
    world.insert_resource(TestUsize(30));

    let mut queue = CommandQueue::default();
    let mut commands = Commands::new(&mut queue, &mut world);

    // method call on Commands
    commands.sub(10);
    // calling method on trait directly
    CommandsSubExt::sub(&mut commands, 5);
    // adding command struct
    commands.add(SubCommand { n: 5 });

    queue.apply(&mut world);

    // method call on World
    world.sub(5);
    // calling method on trait directly
    CommandsSubExt::sub(&mut world, 5);

    assert_eq!(**world.resource::<TestUsize>(), 0);
}

/// The `name` attribute should affect all three ways of calling the entity_command
#[test]
fn bar_becomes_bus() {
    let mut world = World::new();
    let entity = world.spawn(TestUsize(30)).id();

    let mut queue = CommandQueue::default();
    let mut commands = Commands::new(&mut queue, &mut world);

    // method call on Commands
    commands.entity(entity).bus(10);
    // calling method on trait directly
    EntityCommandsBusExt::bus(&mut commands.entity(entity), 5);
    // adding command struct
    commands.entity(entity).add(BusEntityCommand { n: 5 });

    queue.apply(&mut world);

    let mut world_entity = world.entity_mut(entity);

    // method call on EntityWorldMut
    world_entity.bus(5);
    // calling method on trait directly
    EntityCommandsBusExt::bus(&mut world_entity, 5);

    assert_eq!(**world.query::<&TestUsize>().single(&world), 0);
}
