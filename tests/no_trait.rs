use bevy::ecs::system::CommandQueue;
use bevy::prelude::*;
use bevy_commandify::*;

#[command(no_trait)]
fn foo(_world: &mut World) {}

#[entity_command(no_trait)]
fn bar(_world: &mut World, _entity: Entity) {}

#[test]
fn struct_command_still_works() {
    let mut world = World::new();

    let mut queue = CommandQueue::default();
    let mut commands = Commands::new(&mut queue, &mut world);

    commands.add(FooCommand);
    commands.spawn_empty().add(BarEntityCommand);

    queue.apply(&mut world);
}
