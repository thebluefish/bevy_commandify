use bevy::prelude::*;
use bevy_commandify::*;

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
    commands.error_out(false);
    commands.only_ok();
    commands.only_err();
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

fn handle_ok(In(value): In<u32>) {
    println!("Everything is fine. See: {}", value);
}

fn report_error(In(error): In<&'static str>) {
    println!("Error: {}", error);
}

#[command(ok = handle_ok, err = report_error)]
fn error_out(world: &mut World, success: bool) -> Result<u32, &'static str> {
    if success {
        Ok(42)
    } else {
        Err("Something went wrong")
    }
}

#[command(ok = handle_ok)]
fn only_ok(world: &mut World) -> Result<u32, &'static str> {
    Ok(3)
}

#[command(err = report_error)]
fn only_err(world: &mut World) -> Result<u32, &'static str> {
    Err("It's okay to be in error.")
}
