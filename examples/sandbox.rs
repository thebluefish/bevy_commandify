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
