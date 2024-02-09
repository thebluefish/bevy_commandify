use inflector::*;
use proc_macro::TokenStream as ProcTokenStream;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{
    parse_macro_input, parse_quote, Error, Expr, ExprLit, FnArg, GenericParam, ItemFn, Lit, Meta,
    MetaNameValue, Path, ReturnType, Signature, Type,
};

/// Promotes a function to a Command struct, and creates an equivalent Commands method via trait extensions
///
/// - `#[command(no_trait)]` prevents generating a trait method for Commands
/// - `#[command(name = T)]` will use this name for the method and related struct/trait names
/// - `#[command(struct_name = T)]` will use this name for the generated struct, defaults to `<Foo>Command`
/// - `#[command(trait_name = T)]` will use this name for the generated trait, defaults to `Commands<Foo>Ext`
/// - `#[command(ecs = T)]` to change the crate root to T, defaults to `bevy::ecs`
/// - `#[command(bevy_ecs)]` to change the crate root to `bevy_ecs`
///
/// Note: `T`s may be optionally quoted
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
#[proc_macro_attribute]
pub fn entity_command(args: ProcTokenStream, input: ProcTokenStream) -> ProcTokenStream {
    let args = parse_macro_input!(args with Punctuated::<Meta, syn::Token![,]>::parse_terminated);
    let item = parse_macro_input!(input as ItemFn);

    commandify(args, item, true)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

fn commandify(
    args: Punctuated<Meta, syn::Token![,]>,
    item: ItemFn,
    entity_command: bool,
) -> Result<TokenStream, Error> {
    let ItemFn {
        attrs,
        vis,
        sig,
        block,
    } = item;
    let Signature {
        constness,
        asyncness,
        unsafety,
        abi,
        ident,
        generics,
        inputs,
        variadic,
        output,
        ..
    } = sig;

    // general guards
    // I actually have no idea if we should care about this case
    if let Some(variadic) = variadic {
        return Err(Error::new(variadic.span(), "command cannot be variadic"));
    }
    if let ReturnType::Type(_, ty) = output {
        return Err(Error::new(ty.span(), "command cannot define a return type"));
    }

    // attributes
    let mut no_trait = false;
    let mut name = ident;
    let mut struct_name = None;
    let mut trait_name = None;
    let mut ecs_root = None;

    // parse attributes
    for meta in args {
        match meta {
            Meta::Path(path) if path.is_ident("no_trait") => {
                no_trait = true;
            }
            Meta::Path(path) if path.is_ident("bevy_ecs") => {
                ecs_root = Some(parse_quote!(::bevy_ecs));
            }
            Meta::NameValue(MetaNameValue { path, value, .. }) if path.is_ident("name") => {
                name = expr_to_ident(value)?;
            }
            Meta::NameValue(MetaNameValue { path, value, .. }) if path.is_ident("struct_name") => {
                struct_name = Some(expr_to_ident(value)?);
            }
            Meta::NameValue(MetaNameValue { path, value, .. }) if path.is_ident("trait_name") => {
                trait_name = Some(expr_to_ident(value)?);
            }
            Meta::NameValue(MetaNameValue { path, value, .. }) if path.is_ident("ecs") => {
                ecs_root = Some(expr_to_path(value)?);
            }
            _ => {
                return Err(Error::new(
                    meta.span(),
                    format!("Unknown attribute `{}`", meta.to_token_stream()),
                ))
            }
        }
    }

    // generate default names late so that the `name` field applies
    let command_struct = if entity_command {
        "EntityCommand"
    } else {
        "Command"
    };
    let struct_name = struct_name.unwrap_or(Ident::new(
        &format!("{}{command_struct}", name.to_string().to_pascal_case()),
        name.span(),
    ));
    let trait_name = trait_name.unwrap_or(Ident::new(
        &format!("{command_struct}s{}Ext", name.to_string().to_pascal_case()),
        name.span(),
    ));
    let ecs_root = ecs_root.unwrap_or(parse_quote!(::bevy::ecs));

    let mut generic_names = Vec::<TokenStream>::new();
    for param in &generics.params {
        let name = match param {
            GenericParam::Lifetime(inner) => {
                let token = &inner.lifetime;
                quote!(#token)
            }
            GenericParam::Type(inner) => {
                let token = &inner.ident;
                quote!(#token)
            }
            GenericParam::Const(inner) => {
                let token = &inner.ident;
                quote!(#token)
            }
        };
        generic_names.push(name);
    }
    let generic_names = if generic_names.is_empty() {
        quote!()
    } else {
        quote!(< #(#generic_names,)* >)
    };

    let mut fields = Vec::<TokenStream>::new();
    let mut struct_fields = Vec::<TokenStream>::new();
    let mut field_names = Vec::<TokenStream>::new();
    let mut world_field = None;
    let mut entity_field = None;
    for input in inputs {
        match input {
            // `self` types smell of methods
            FnArg::Receiver(inner) => {
                return Err(Error::new(inner.span(), "Commands cannot be methods"))
            }
            FnArg::Typed(pt) => {
                let name = &pt.pat;
                // find `&World` and `Entity` types
                match pt.ty.as_ref() {
                    Type::Reference(tr) => {
                        if tr.elem.to_token_stream().to_string() == "World" {
                            world_field = Some(quote!(#pt));
                            continue;
                        }
                    }
                    Type::Path(path) => {
                        if entity_command && path.to_token_stream().to_string() == "Entity" {
                            entity_field = Some(quote!(#pt));
                            continue;
                        }
                    }
                    _ => (),
                }
                fields.push(quote!(#pt ,));
                struct_fields.push(quote!(pub #pt ,));
                field_names.push(quote!(#name ,));
            }
        }
    }

    if world_field.is_none() {
        return Err(Error::new(
            Span::call_site(),
            "Commands must take in a `&mut World` parameter",
        ));
    }
    if entity_command && entity_field.is_none() {
        return Err(Error::new(
            Span::call_site(),
            "Entity commands must take in a `Entity` parameter",
        ));
    }

    let field_frag = if fields.is_empty() {
        quote!( ; )
    } else {
        quote!( { #(#struct_fields)* } )
    };

    let apply_params = if entity_command {
        quote!((self, #entity_field, #world_field))
    } else {
        quote!((self, #world_field))
    };

    let command_trait = if entity_command {
        quote!(EntityCommand)
    } else {
        quote!(Command)
    };
    let impl_command_frag = quote!(
        impl #generics #ecs_root ::system:: #command_trait for #struct_name #generic_names {
            fn apply #apply_params {
                let #struct_name {#(#field_names)*} = self;
                #block
            }
        }
    );

    let commands_trait_frag = if no_trait {
        quote!()
    } else {
        let commands_struct = if entity_command {
            quote!(EntityCommands<'_, '_, '_>)
        } else {
            quote!(Commands<'_, '_>)
        };
        quote!(
            pub trait #trait_name {
                #(#attrs)*
                fn #name #generics (&mut self, #(#fields)*);
            }

            impl #trait_name for #ecs_root ::system:: #commands_struct {
                fn #name #generics (&mut self, #(#fields)*) {
                    self.add(#struct_name {#(#field_names)*});
                }
            }
        )
    };

    Ok(quote!(
        #(#attrs)*
        #vis
        #constness
        #asyncness
        #unsafety
        #abi
        struct
        #struct_name
        #generics
        #field_frag
        #impl_command_frag
        #commands_trait_frag
    ))
}

/// Grabs the T as a syn::ident from either `T` or `"T"`
fn expr_to_ident(expr: Expr) -> Result<Ident, Error> {
    let ident = match expr {
        Expr::Lit(ExprLit {
            lit: Lit::Str(lit), ..
        }) => lit.parse()?,
        Expr::Path(mut path) => {
            if path.path.segments.len() < 1 {
                return Err(Error::new(path.span(), "Name must exist"));
            }
            if path.path.segments.len() > 1 {
                return Err(Error::new(path.span(), "Name must be an ident, found path"));
            }
            path.path.segments.pop().unwrap().into_value().ident
        }
        value => {
            return Err(Error::new(
                value.span(),
                format!("invalid name: `{}`", value.to_token_stream()),
            ))
        }
    };
    Ok(ident)
}

/// Grabs the T as a syn::Path from either `T` or `"T"`
fn expr_to_path(expr: Expr) -> Result<Path, Error> {
    let path = match expr {
        Expr::Lit(ExprLit {
            lit: Lit::Str(lit), ..
        }) => lit.parse_with(Path::parse_mod_style)?,
        Expr::Path(path) => path.path,
        value => {
            return Err(Error::new(
                value.span(),
                format!("invalid path: `{}`", value.to_token_stream()),
            ))
        }
    };
    Ok(path)
}
