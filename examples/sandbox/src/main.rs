use bevy_derive_commands::*;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_systems(Update, (setup, apply_deferred, check).chain())
        .run();
}

fn setup(mut commands: Commands) {
    commands.create_stuff(TransformBundle::default(), 3);
}

fn check(query: Query<Entity, With<Transform>>) {
    for item in &query {
        println!("got {item:?}");
    }
}

/// A command that spawns a bundle `n` times
#[command]
fn create_stuff<B: Bundle + Clone>(world: &mut World, bundle: B, times: usize) {
    for _ in 0..times {
        world.spawn(bundle.clone());
    }
}