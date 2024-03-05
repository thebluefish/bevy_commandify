use bevy::ecs::system::CommandQueue;
use bevy::prelude::*;
use bevy_commandify::*;

#[derive(Resource)]
struct OkHandled;

#[derive(Resource)]
struct ErrorHandled;

fn handle_ok(In(_): In<bool>, mut commands: Commands) {
    commands.insert_resource(OkHandled);
}

fn handle_err(In(_): In<bool>, mut commands: Commands) {
    commands.insert_resource(ErrorHandled);
}

#[command(ok = handle_ok, err = handle_err)]
fn test_handler(world: &mut World, success: bool) -> Result<bool, bool> {
    if success {
        Ok(true)
    } else {
        Err(false)
    }
}

#[test]
fn error_handler_handles_err() {
    let mut world = World::new();

    let mut queue = CommandQueue::default();
    let mut commands = Commands::new(&mut queue, &world);

    commands.add(TestHandlerCommand { success: false });

    queue.apply(&mut world);

    assert!(world.get_resource::<ErrorHandled>().is_some());
}

#[test]
fn error_handler_handles_ok() {
    let mut world = World::new();

    let mut queue = CommandQueue::default();
    let mut commands = Commands::new(&mut queue, &world);

    commands.add(TestHandlerCommand { success: true });

    queue.apply(&mut world);

    assert!(world.get_resource::<OkHandled>().is_some());
}
