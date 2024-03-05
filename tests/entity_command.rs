use bevy::ecs::system::CommandQueue;
use bevy::prelude::*;
use bevy_commandify::*;

#[derive(Component)]
struct Marker;

#[entity_command]
fn apply_marker(id: Entity, world: &mut World) {
    world.entity_mut(id).insert(Marker);
}

#[test]
fn marker_applied() {
    let mut world = World::new();

    let mut queue = CommandQueue::default();
    let mut commands = Commands::new(&mut queue, &world);

    let mut entity_commands = commands.spawn_empty();
    let id = entity_commands.id();
    entity_commands.apply_marker();

    queue.apply(&mut world);

    assert!(world.entity(id).contains::<Marker>());
}
