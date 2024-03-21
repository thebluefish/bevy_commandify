use bevy::ecs::system::CommandQueue;
use bevy::prelude::*;
use bevy_commandify::*;

const TEST_VALUE: i32 = 33;

#[derive(Resource)]
struct OutputPiped;

fn process_input(In(value): In<i32>, mut commands: Commands) {
    if value == TEST_VALUE {
        commands.insert_resource(OutputPiped);
    }
}

#[command(pipe = process_input)]
fn test_pipe(world: &mut World, value: i32) -> i32 {
    return value;
}

#[test]
fn can_pipe_output() {
    let mut world = World::new();

    let mut queue = CommandQueue::default();
    let mut commands = Commands::new(&mut queue, &world);

    commands.add(TestPipeCommand { value: TEST_VALUE });

    queue.apply(&mut world);

    assert!(world.get_resource::<OutputPiped>().is_some());
}
