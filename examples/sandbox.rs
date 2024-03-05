use bevy::prelude::*;
use bevy_commandify::*;
use bevy_ecs::system::RunSystemOnce;

fn main() {
    App::new()
        .add_systems(Update, (setup, apply_deferred, check).chain())
        .run();
}

fn setup(mut commands: Commands) {
    commands.add(CreateStuffCommand {
        bundle: TransformBundle::default(),
        n: 3,
    });
    commands.create_stuff(TransformBundle::default(), 3);
    commands.error_out(true);
}

fn check(query: Query<Entity, With<Transform>>) {
    for item in &query {
        println!("got {item:?}");
    }
}

/// A command that spawns a bundle `n` times
#[command]
fn create_stuff<B: Bundle + Clone>(world: &mut World, bundle: B, n: usize) {
    for _ in 0..n {
        world.spawn(bundle.clone());
    }
}

fn report_error(In(error): In<&'static str>) {
    println!("Handling an Error: {:?}", error);
}

#[command(err = report_error)]
fn error_out(world: &mut World, success: bool) -> Result<(), &'static str> {
    if success {
        println!("Everything is fine");
        Ok(())
    } else {
        Err("Something went wrong")
    }
}
