use bevy::ecs::system::CommandQueue;
use bevy::prelude::*;
use bevy_commandify::*;

mod common;
use common::TestUsize;

#[command(struct_name = Foo)]
fn add(world: &mut World, n: usize) {
    let mut m = world.resource_mut::<TestUsize>();
    **m += n;
}

#[entity_command(name = do_sub, struct_name = "Bar")]
fn sub(world: &mut World, entity: Entity, n: usize) {
    let mut m = world
        .query::<&mut TestUsize>()
        .get_mut(world, entity)
        .unwrap();
    **m -= n;
}

/// We should be able to add our entity_command via the defined `struct_name`
#[test]
fn renamed_struct() {
    let mut world = World::new();
    world.insert_resource(TestUsize(10));

    let mut queue = CommandQueue::default();
    let mut commands = Commands::new(&mut queue, &mut world);

    commands.add(Foo { n: 10 });

    queue.apply(&mut world);

    assert_eq!(**world.resource::<TestUsize>(), 20);
}

/// We should be able to add our entity_command via the defined `struct_name`
#[test]
fn renamed_entity_struct() {
    let mut world = World::new();
    let entity = world.spawn(TestUsize(20)).id();

    let mut queue = CommandQueue::default();
    let mut commands = Commands::new(&mut queue, &mut world);

    commands.entity(entity).add(Bar { n: 10 });
    commands.entity(entity).do_sub(10);

    queue.apply(&mut world);

    assert_eq!(**world.query::<&TestUsize>().single(&world), 0);
}
