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

#[command(name = do_sub, trait_name = "BarExt")]
fn sub(world: &mut World, n: usize) {
    let mut m = world.resource_mut::<TestUsize>();
    **m -= n;
}

/// The `trait_name` should define the generated trait name
#[test]
fn renamed_structs() {
    let mut world = World::new();
    world.insert_resource(TestUsize(10));

    let mut queue = CommandQueue::default();
    let mut commands = Commands::new(&mut queue, &mut world);

    FooExt::add(&mut commands, 10);
    BarExt::do_sub(&mut commands, 10);
    commands.do_sub(10);

    queue.apply(&mut world);

    assert_eq!(**world.resource::<TestUsize>(), 0);
}
