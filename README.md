Derive macro for creating bevy `Commands` methods.

## Example

```rust
/// A command that spawns a bundle `n` times
#[command]
fn create_stuff(world: &mut World, bundle: B, n: usize) {
    for _ in 0..times {
        world.spawn(bundle.clone());
    }
}

fn setup(mut commands: Commands) {
    // Use the generated method
    commands.create_stuff(TransformBundle::default(), 3);
    // Or add the generated command type directly
    commands.add(CreateStuffCommand { bundle: TransformBundle::default(), n: 3 });
}
```

### Compatibility

| Bevy   | Crate |
|--------|-------|
| `0.12` | `0.1` |