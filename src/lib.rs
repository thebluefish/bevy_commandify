mod gen;
mod parse;

use gen::commandify;

use proc_macro::TokenStream as ProcTokenStream;
use syn::punctuated::Punctuated;
use syn::{parse_macro_input, Error, ItemFn, Meta};

/// Promotes a function to a `Command` struct, and creates an equivalent `Commands` and `World` method via trait extensions
///
/// - `#[command(no_trait)]` prevents generating a trait method for `Commands`
/// - `#[command(no_world)]` prevents generating a trait impl for `World`
/// - `#[command(name = T)]` will use this name for the method and related struct/trait names
/// - `#[command(struct_name = T)]` will use this name for the generated struct, defaults to `<Foo>Command`
/// - `#[command(trait_name = T)]` will use this name for the generated trait, defaults to `Commands<Foo>Ext`
/// - `#[command(ecs = T)]` to change the crate root to T, defaults to `bevy::ecs`
/// - `#[command(bevy_ecs)]` to change the crate root to `bevy_ecs`
///
/// Note: `T`s may be optionally quoted
///
/// Commands may optionally return `&mut Self` to allow chaining their calls
#[proc_macro_attribute]
pub fn command(args: ProcTokenStream, input: ProcTokenStream) -> ProcTokenStream {
    let args = parse_macro_input!(args with Punctuated::<Meta, syn::Token![,]>::parse_terminated);
    let item = parse_macro_input!(input as ItemFn);

    commandify(args, item, false)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

/// Promotes a function to an EntityCommand struct, and creates an equivalent EntityCommands method via trait extensions
///
/// - `#[entity_command(no_trait)]` prevents generating a trait method for EntityCommands
/// - `#[entity_command(name = T)]` will use this name for the method and related struct/trait names
/// - `#[entity_command(struct_name = T)]` will use this name for the generated struct, defaults to `<Foo>EntityCommand`
/// - `#[entity_command(trait_name = T)]` will use this name for the generated trait, defaults to `EntityCommands<Foo>Ext`
/// - `#[entity_command(ecs = T)]` to change the crate root to T, defaults to `bevy::ecs`
/// - `#[entity_command(bevy_ecs)]` to change the crate root to `bevy_ecs`
///
/// Note: `T`s may be optionally quoted
///
/// Commands may optionally return `&mut Self` to allow chaining their calls
#[proc_macro_attribute]
pub fn entity_command(args: ProcTokenStream, input: ProcTokenStream) -> ProcTokenStream {
    let args = parse_macro_input!(args with Punctuated::<Meta, syn::Token![,]>::parse_terminated);
    let item = parse_macro_input!(input as ItemFn);

    commandify(args, item, true)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}
