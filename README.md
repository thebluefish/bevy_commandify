[![Crates.io](https://img.shields.io/crates/v/bevy_commandify.svg)](https://crates.io/crates/bevy_commandify)
[![Docs](https://docs.rs/bevy_commandify/badge.svg)](https://docs.rs/bevy_commandify/latest/bevy_commandify/)

Derive macro for creating bevy `Commands` methods from functions.

## Example

```rust
use bevy_commandify::*;

#[command]
fn foo(world: &mut World) {
    // Bear in mind that Commands *defer* work
    // This function will not be called immediately
    // It will be executed either at the end of the current schedule or when `apply_deferred` is next called within the current schedule
}

fn setup(mut commands: Commands) {
    // Fire our command directly
    commands.foo();
    // call it via the generated extension trait
    CommandsFooExt::foo(&mut commands);
    // Add the command as a struct
    commands.add(FooCommand);
}

```

See also [the example](/examples/sandbox/src/main.rs) and [tests](/tests)


### Attributes

- `#[command(no_trait)]` prevents generating a trait method for Commands, but will still generate a `Command` struct you can add:
```rust
#[command(no_trait)]
fn foo(world: &mut World) { }

commands.foo(); // This will throw an error
commands.add(FooCommand); // This will still work
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

- `#[command(ecs = T)]` or `#[command(bevy_ecs)]` to point the macro to the correct bevy crate if you don't use `bevy` directly

### Compatibility

| Bevy   | Crate |
|--------|-------|
| `0.12` | `0.1` |