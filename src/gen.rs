use crate::parse::ExprExt;
use inflector::*;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{
    parse_quote, Error, FnArg, GenericParam, ItemFn, Meta, MetaNameValue, ReturnType, Signature,
    Type,
};

pub(crate) fn commandify(
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

    // I actually have no idea if we should care about this case
    if let Some(variadic) = variadic {
        return Err(Error::new(variadic.span(), "command cannot be variadic"));
    }

    let do_return = match &output {
        ReturnType::Type(_, ty) => match ty.as_ref() {
            // find optional `&mut Self` return type
            Type::Reference(tr)
                if tr.mutability.is_some() && tr.elem.to_token_stream().to_string() == "Self" =>
            {
                true
            }
            _ => {
                return Err(Error::new(
                    ty.span(),
                    "command may not define a return type, except for `&mut Self`",
                ))
            }
        },
        _ => false,
    };

    // attributes
    let mut no_trait = false;
    let mut no_world = false;
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
            Meta::Path(path) if path.is_ident("no_world") => {
                no_world = true;
            }
            Meta::Path(path) if path.is_ident("bevy_ecs") => {
                ecs_root = Some(parse_quote!(::bevy_ecs));
            }
            Meta::NameValue(MetaNameValue { path, value, .. }) if path.is_ident("name") => {
                name = value.try_to_ident()?;
            }
            Meta::NameValue(MetaNameValue { path, value, .. }) if path.is_ident("struct_name") => {
                struct_name = Some(value.try_to_ident()?);
            }
            Meta::NameValue(MetaNameValue { path, value, .. }) if path.is_ident("trait_name") => {
                trait_name = Some(value.try_to_ident()?);
            }
            Meta::NameValue(MetaNameValue { path, value, .. }) if path.is_ident("ecs") => {
                ecs_root = Some(value.try_to_path()?);
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

    let return_frag = if do_return { quote!(self) } else { quote!() };

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
                fn #name #generics (&mut self, #(#fields)*) #output;
            }

            impl #trait_name for #ecs_root ::system:: #commands_struct {
                fn #name #generics (&mut self, #(#fields)*) #output {
                    self.add(#struct_name {#(#field_names)*});
                    #return_frag
                }
            }
        )
    };

    let impl_world_frag = if no_trait || no_world {
        quote!()
    } else if entity_command {
        quote!(
            impl #trait_name for #ecs_root ::world::EntityWorldMut<'_> {
                fn #name #generics (&mut self, #(#fields)*) #output {
                    let id = self.id();
                    self.world_scope(|world| {
                        <#struct_name #generic_names as #ecs_root ::system:: #command_trait>::apply (#struct_name {#(#field_names)*}, id, world);
                    });
                    #return_frag
                }
            }
        )
    } else {
        quote!(
            impl #trait_name for #ecs_root ::world::World {
                fn #name #generics (&mut self, #(#fields)*)  #output {
                    <#struct_name #generic_names as #ecs_root ::system:: #command_trait>::apply (#struct_name {#(#field_names)*}, self);
                    #return_frag
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
        #impl_world_frag
    ))
}
