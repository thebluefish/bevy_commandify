use bevy::prelude::*;
use bevy_commandify::*;
use bevy_ecs::schedule::ScheduleLabel;
use bevy_ecs::system::CommandQueue;

mod common;
use common::TestUsize;

#[command]
fn foo(In(n): In<usize>, mut m: ResMut<TestUsize>) {
    **m -= n;
}

#[command]
fn irony(mut commands: Commands) -> &mut Self {
    commands.foo(5);
}

#[command]
fn one(In(mut stuff): In<(usize, usize)>, mut item: ResMut<TestUsize>) -> &mut Self {
    stuff.0 *= 2;
    stuff.1 += stuff.0;
    **item -= stuff.1;
}

#[entity_command]
fn two(In((entity, mut n)): In<(Entity, usize)>, mut query: Query<&mut TestUsize>) {
    n *= 2;
    let mut m = query.get_mut(entity).unwrap();
    **m -= n;
}

#[derive(ScheduleLabel, Hash, Debug, Eq, PartialEq, Clone)]
struct TestSchedule;

/// all three ways of calling the command
#[test]
fn command() {
    let mut world = World::new();
    world.insert_resource(TestUsize(30));

    let mut schedule = Schedule::new(TestSchedule);
    schedule.add_systems(irony);

    // method call as a normal bevy system
    schedule.run(&mut world);

    let mut queue = CommandQueue::default();
    let mut commands = Commands::new(&mut queue, &mut world);

    // subtract 5 for irony and (5*2)+0 for one
    commands.irony().one((5, 0));

    queue.apply(&mut world);

    // subtract 5 for irony and 5 for one
    world.irony().one((0, 5));

    assert_eq!(**world.resource::<TestUsize>(), 0);
}

/// all three ways of calling the entity_command
#[test]
fn entity_command() {
    let mut world = World::new();
    let entity = world.spawn(TestUsize(30)).id();

    let mut queue = CommandQueue::default();
    let mut commands = Commands::new(&mut queue, &mut world);

    // method call on Commands
    commands.entity(entity).two(5);

    queue.apply(&mut world);

    let mut world_entity = world.entity_mut(entity);

    world_entity.two(5);
    EntityCommandsTwoExt::two(&mut world_entity, 5);

    assert_eq!(**world.query::<&TestUsize>().single(&world), 0);
}
