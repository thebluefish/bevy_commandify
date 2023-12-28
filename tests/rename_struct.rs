use bevy_commandify::*;
use bevy::prelude::*;
use bevy::ecs::system::CommandQueue;

mod common;
use common::TestUsize;

#[command(struct_name = Foo)]
fn add(world: &mut World, n: usize) {
    let mut m = world.resource_mut::<TestUsize>();
    **m += n;
}

#[command(name = do_sub, struct_name = "Bar")]
fn sub(world: &mut World, n: usize) {
    let mut m = world.resource_mut::<TestUsize>();
    **m -= n;
}

/// The `struct_name` should define the generated struct name
#[test]
fn renamed_structs() {
    let mut world = World::new();
    world.insert_resource(TestUsize(10));

    let mut queue = CommandQueue::default();
    let mut commands = Commands::new(&mut queue, &mut world);

    commands.add(Foo { n: 10 });
    commands.add(Bar { n: 10 });
    commands.do_sub(10);

    queue.apply(&mut world);

    assert_eq!(**world.resource::<TestUsize>(), 0);
}