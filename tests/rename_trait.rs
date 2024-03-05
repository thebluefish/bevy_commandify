use bevy::ecs::system::CommandQueue;
use bevy::prelude::*;
use bevy_commandify::*;

mod common;
use common::TestUsize;

#[command(trait_name = FooExt)]
fn add(world: &mut World, n: usize) {
    let mut m = world.resource_mut::<TestUsize>();
    **m += n;
}

#[entity_command(name = do_sub, trait_name = "BarExt")]
fn sub(world: &mut World, entity: Entity, n: usize) {
    let mut m = world
        .query::<&mut TestUsize>()
        .get_mut(world, entity)
        .unwrap();
    **m -= n;
}

/// We should be able to call our command via the defined `trait_name`
#[test]
fn renamed_trait() {
    let mut world = World::new();
    world.insert_resource(TestUsize(10));

    let mut queue = CommandQueue::default();
    let mut commands = Commands::new(&mut queue, &world);

    // Call via Commands
    FooExt::add(&mut commands, 10);

    queue.apply(&mut world);

    // Call via World
    FooExt::add(&mut world, 10);

    assert_eq!(**world.resource::<TestUsize>(), 30);
}

/// We should be able to call our entity_command via the defined `trait_name`
#[test]
fn renamed_entity_trait() {
    let mut world = World::new();
    let entity = world.spawn(TestUsize(30)).id();

    let mut queue = CommandQueue::default();
    let mut commands = Commands::new(&mut queue, &world);

    // Call via Commands
    BarExt::do_sub(&mut commands.entity(entity), 10);
    commands.entity(entity).do_sub(10);

    queue.apply(&mut world);

    let mut world_entity = world.entity_mut(entity);

    // Call via World
    BarExt::do_sub(&mut world_entity, 5);
    world_entity.do_sub(5);

    assert_eq!(**world.query::<&TestUsize>().single(&world), 0);
}
