use crate::parse;
use crate::parse::{MacroArgs, SysArgs, SystemArgs};
use inflector::*;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{parse_quote, Error, GenericParam, ItemFn, Meta, Signature};

pub fn commandify(
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
        fn_token,
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

    // parse return argument
    let do_return = parse::return_type(&output)?;

    // parse macro args
    let MacroArgs {
        no_trait,
        no_world,
        name,
        struct_name,
        trait_name,
        ecs_root,
        error_handler,
    } = parse::macro_args(&args, ident.clone())?;

    // generate default names late so that the `name` field applies
    let command_struct = if entity_command {
        "EntityCommand"
    } else {
        "Command"
    };
    let struct_name = struct_name.unwrap_or_else(|| {
        Ident::new(
            &format!("{}{command_struct}", name.to_string().to_pascal_case()),
            name.span(),
        )
    });
    let trait_name = trait_name.unwrap_or_else(|| {
        Ident::new(
            &format!("{command_struct}s{}Ext", name.to_string().to_pascal_case()),
            name.span(),
        )
    });
    let ecs_root = ecs_root.unwrap_or_else(|| parse_quote!(::bevy::ecs));

    // parse generics
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

    // parse doc comments
    let docs = parse::docs(&attrs);

    // parse fn args
    let SysArgs {
        entity,
        fields,
        def_field_names,
        impl_field_names,
        args,
    } = parse::fn_args(&inputs, entity_command)?;

    if entity_command && entity.is_none() {
        return Err(Error::new(
            Span::call_site(),
            "Entity commands must take in a `Entity` parameter",
        ));
    }

    // generate fragments to be combined later

    let generic_names = if generic_names.is_empty() {
        quote!()
    } else {
        quote!(< #(#generic_names,)* >)
    };

    // piece back the original system sans return type
    let mut fn_frag = quote!(
        #[allow(unused)]
        #(#attrs)*
        #vis
        #constness
        #asyncness
        #unsafety
        #abi
        #fn_token
        #ident
        #generics
        (#inputs)
        #variadic
        #block
    );

    let (block, fn_error_handler_frag) = if let Some(error_handler) = error_handler {
        // Create a new ident which is the old fn name with _error_handled appended.
        let error_handler_ident = Ident::new(&format!("{}_error_handled", ident), ident.span());

        let error_handling_block = quote!({
            let result = #error_handler_ident(world);
            if let Err(error) = result {
                world.run_system_once_with(error, #error_handler);
            }
        });

        fn_frag = quote!(
            #[allow(unused)]
            #(#attrs)*
            #vis
            #constness
            #asyncness
            #unsafety
            #abi
            #fn_token
            #error_handler_ident
            #generics
            (#inputs)
            #variadic
            #output
            #block
        );

        (
            error_handling_block.clone(),
            quote!(
                #[allow(unused)]
                #(#attrs)*
                #vis
                #constness
                #asyncness
                #unsafety
                #abi
                #fn_token
                #ident
                #generics
                (#inputs)
                #variadic
                #error_handling_block
            ),
        )
    } else {
        (quote!(#block), quote!())
    };

    // which trait we're implementing for
    let command_trait = if entity_command {
        quote!(EntityCommand)
    } else {
        quote!(Command)
    };

    let return_frag = if do_return { quote!(self) } else { quote!() };
    let output_frag = if do_return { quote!(#output) } else { quote!() };

    // the fields of our generated struct
    let struct_fields_frag = if fields.is_empty() {
        quote!( ; )
    } else {
        quote!( { #(pub #fields,)* } )
    };

    // The inputs passed to our system
    let system_in_frag = match &args {
        SystemArgs::Exclusive { .. } => quote!(),
        SystemArgs::System { systems_in, .. } => {
            if systems_in.len() > 1 {
                quote!((#(#systems_in,)*))
            } else if let Some(field) = systems_in.last() {
                quote!(#field)
            } else {
                quote!()
            }
        }
    };

    // Generates a `Commands` or `EntityCommands` impl for our struct
    let impl_command_frag = match &args {
        SystemArgs::Exclusive { world } => {
            let apply_params = if entity_command {
                quote!((self, #entity, #world))
            } else {
                quote!((self, #world))
            };

            quote!(
                impl #generics #ecs_root ::system:: #command_trait for #struct_name #generic_names {
                    fn apply #apply_params {
                        let #struct_name {#(#impl_field_names,)*} = self;
                        #block
                    }
                }
            )
        }
        SystemArgs::System { .. } => {
            let apply_params = if entity_command {
                quote!((self, #entity, world: &mut #ecs_root ::world::World))
            } else {
                quote!((self, world: &mut #ecs_root ::world::World))
            };
            if fields.is_empty() {
                quote!(
                    impl #generics #ecs_root ::system:: #command_trait for #struct_name #generic_names {
                        fn apply #apply_params {
                            use #ecs_root ::system::RunSystemOnce;
                            world.run_system_once(#ident);
                        }
                    }
                )
            } else {
                quote!(
                    impl #generics #ecs_root ::system:: #command_trait for #struct_name #generic_names {
                        fn apply #apply_params {
                            use #ecs_root ::system::RunSystemOnce;
                            let #struct_name {#(#def_field_names,)*} = self;
                            world.run_system_once_with(#system_in_frag, #ident);
                        }
                    }
                )
            }
        }
    };

    // Generates a new trait + method for issuing our command
    // Implements this new trait for `Commands` or `EntityCommands`
    let commands_trait_frag = match &args {
        SystemArgs::Exclusive { .. } => {
            if no_trait {
                quote!()
            } else {
                let commands_struct = if entity_command {
                    quote!(EntityCommands<'_>)
                } else {
                    quote!(Commands<'_, '_>)
                };
                quote!(
                    pub trait #trait_name {
                        #docs
                        fn #name #generics (&mut self, #(#fields,)*) #output_frag;
                    }

                    impl #trait_name for #ecs_root ::system:: #commands_struct {
                        fn #name #generics (&mut self, #(#fields,)*) #output_frag {
                            self.add(#struct_name {#(#def_field_names,)*});
                            #return_frag
                        }
                    }
                )
            }
        }
        SystemArgs::System { .. } => {
            if no_trait {
                quote!()
            } else {
                let commands_struct = if entity_command {
                    quote!(EntityCommands<'_>)
                } else {
                    quote!(Commands<'_, '_>)
                };

                quote!(
                    pub trait #trait_name {
                        #docs
                        fn #name #generics (&mut self #(,#fields,)*) #output_frag;
                    }

                    impl #trait_name for #ecs_root ::system:: #commands_struct {
                        fn #name #generics (&mut self #(,#fields,)*) #output_frag {
                            self.add(#struct_name {#(#def_field_names,)*});
                            #return_frag
                        }
                    }
                )
            }
        }
    };

    // Implements the same trait as above, but for `World` or `EntityWorldMut`
    let impl_world_frag = match &args {
        SystemArgs::Exclusive { .. } => {
            if no_trait || no_world {
                quote!()
            } else if entity_command {
                quote!(
                    impl #trait_name for #ecs_root ::world::EntityWorldMut<'_> {
                        fn #name #generics (&mut self, #(#fields,)*) #output_frag {
                            let id = self.id();
                            self.world_scope(|world| {
                                <#struct_name #generic_names as #ecs_root ::system:: #command_trait>::apply (#struct_name {#(#def_field_names,)*}, id, world);
                            });
                            #return_frag
                        }
                    }
                )
            } else {
                quote!(
                    impl #trait_name for #ecs_root ::world::World {
                        fn #name #generics (&mut self, #(#fields,)*) #output_frag {
                            <#struct_name #generic_names as #ecs_root ::system:: #command_trait>::apply (#struct_name {#(#def_field_names,)*}, self);
                            #return_frag
                        }
                    }
                )
            }
        }
        SystemArgs::System { entity_name, .. } => {
            let root = if entity_command {
                quote!(#ecs_root ::world::EntityWorldMut<'_>)
            } else {
                quote!(#ecs_root ::world::World)
            };

            let entity_frag = if let Some(entity) = entity_name {
                quote!(let #entity = self.id();)
            } else {
                quote!()
            };

            let run_frag = if entity_command {
                quote!(
                    self.world_scope(|world| {
                        world.run_system_once_with(#system_in_frag, #ident);
                    });
                )
            } else {
                quote!(self.run_system_once_with(#system_in_frag, #ident);)
            };

            if no_trait || no_world {
                quote!()
            } else if fields.is_empty() {
                quote!(
                    impl #trait_name for #root {
                        fn #name #generics (&mut self) #output_frag {
                            use ::bevy::ecs::system::RunSystemOnce;
                            self.run_system_once(#ident);
                            #return_frag
                        }
                    }
                )
            } else {
                quote!(
                    impl #trait_name for #root {
                        fn #name #generics (&mut self #(,#fields)*) #output_frag {
                            use ::bevy::ecs::system::RunSystemOnce;
                            #entity_frag
                            #run_frag
                            #return_frag
                        }
                    }
                )
            }
        }
    };

    Ok(quote!(
        #fn_frag
        #fn_error_handler_frag
        #(#attrs)*
        #vis
        #constness
        #asyncness
        #unsafety
        #abi
        struct
        #struct_name
        #generics
        #struct_fields_frag
        #impl_command_frag
        #commands_trait_frag
        #impl_world_frag
    ))
}
