[![Crates.io](https://img.shields.io/crates/v/bevy_commandify.svg)](https://crates.io/crates/bevy_commandify)
[![Docs](https://docs.rs/bevy_commandify/badge.svg)](https://docs.rs/bevy_commandify/latest/bevy_commandify/)

A macro for creating bevy `Commands` and `EntityCommands` methods from functions.

## Examples

```rust
use bevy_commandify::*;

#[command]
fn foo(world: &mut World, n: usize) {
    // Bear in mind that Commands *defer* work
    // This function will not be called immediately
    // It will be executed at the end of the current schedule 
    // Or when `apply_deferred` is next called within the current schedule
    let mut bar = world.resource_mut::<Bar>();
    **bar -= n;
}

/// Regular bevy systems can be used as commands, too
#[command]
fn irony(mut commands: Commands) {
    commands.foo(5);
}

#[entity_command]
/// Commands may optionally return `&mut Self` to allow chaining command calls
fn bar(world: &mut World, entity: Entity, n: usize) -> &mut Self {
    let mut component = world
        .query::<&mut Bar>()
        .get_mut(world, entity)
        .unwrap();
    **component -= n;
}

fn commands(mut commands: Commands) {
    // Fire our command directly
    commands.foo(10);
    // call it via the generated extension trait
    CommandsFooExt::foo(&mut commands, 10);
    // Add the command as a struct
    commands.add(FooCommand { n: 10 });
}

// Commands may also run against the world directly
fn exclusive_commands(world: &mut World) {
    world.foo(10);
    CommandsFooExt::foo(world, 10);
}

fn entity_commands(mut commands: Commands) {
    let entity = commands.spawn(Bar::default()).id();
    
    commands.entity(entity).bar(10).insert(Foo); // We can chain calls to other commands
    EntityCommandsBarExt::bar(&mut commands.entity(entity), 10);
    commands.entity(entity).add(BarEntityCommand { n: 10 });
}

fn exclusive_entity_commands(world: &mut World) {
    let entity = world.spawn(Bar(0)).id();
    
    world.entity_mut(entity).bar(10);
    EntityCommandsBarExt::bar(&mut world.entity_mut(entity), 10);
}
```

See also [the example](/examples/sandbox/src/main.rs) and [tests](/tests)


### Attributes

The following attributes apply equally to `#[command]` and `#[entity_command]`

- `#[command(no_trait)]` prevents generating a trait method for Commands, but will still generate a `Command` struct you can add:
```rust
#[command(no_trait)]
fn foo(world: &mut World) { }

commands.foo(); // This will throw an error
commands.add(FooCommand); // This will still work
```

- `#[command(no_world)]` prevents generating a trait implementation against `World` or `EntityWorldMut`
```rust
#[command(no_trait)]
fn foo(world: &mut World) { }

world.foo(); // This will throw an error
commands.foo(); // This will still work
```

- `#[command(name = T)]` will use `T` for the generated method and related struct/trait names:
```rust
#[command(name = "bar")]
fn foo(world: &mut World) { }

commands.bar();
CommandsBarExt::bar(&mut commands);
commands.add(BarCommand);
```

- `#[command(struct_name = T)]` will use this name for the generated struct:
```rust
#[command(struct_name = "Bar")]
fn foo(world: &mut World) { }

commands.foo();
CommandsFooExt::foo(&mut commands);
commands.add(Bar);
```

- `#[command(trait_name = T)]` will use this name for the generated trait:
```rust
#[command(trait_name = "BarExt")]
fn foo(world: &mut World) { }

commands.foo();
BarExt::foo(&mut commands);
commands.add(FooCommand);
```

- `#[command(ecs = T)]` or `#[command(bevy_ecs)]` to point the macro to the correct bevy crate if you don't use `bevy` directly.

---

### Compatibility

| Bevy   | Crate        |
|--------|--------------|
| `0.12` | `0.1`, `0.2` |