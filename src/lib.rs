use proc_macro::{TokenStream as ProcTokenStream};
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{parse_macro_input, ItemFn, Signature, ReturnType, FnArg, Type, Item, DeriveInput, Data, Fields, GenericParam};
use inflector::*;

/// Promotes a function to a Command struct, and creates an equivalent Commands method via trait extensions
#[proc_macro_attribute]
pub fn command(_args: ProcTokenStream, input: ProcTokenStream) -> ProcTokenStream {
    let ret = input.clone();
    let input = parse_macro_input!(input as Item);

    match input {
        Item::Fn(ItemFn { attrs, vis, sig, block }) => {
            let Signature { constness, asyncness, unsafety, abi, ident, generics, inputs, variadic, output, .. } = sig;

            let cmd_name = Ident::new(&format!("{}Command", ident.to_string().to_pascal_case()), Span::call_site());
            let trait_name = Ident::new(&format!("Commands{}Ext", ident.to_string().to_pascal_case()), Span::call_site());
            let method_name = Ident::new(&ident.to_string(), Span::call_site());

            if variadic.is_some() {
                panic!("commands cannot be variadic");
            }
            if output != ReturnType::Default {
                panic!("commands cannot define a return type");
            }

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
            }
            else {
                quote!(< #(#generic_names,)* >)
            };

            let mut fields = Vec::<TokenStream>::new();
            let mut field_names = Vec::<TokenStream>::new();
            let mut world_field = None;
            for input in inputs {
                match input {
                    // skip `self` types
                    FnArg::Receiver(_) => {}
                    FnArg::Typed(pt) => {
                        let name = &pt.pat;
                        // find `&World` types
                        if let Type::Reference(tr) = pt.ty.as_ref() {
                            if tr.elem.to_token_stream().to_string() == "World" {
                                world_field = Some(quote!(#pt));
                                continue
                            }
                        }
                        fields.push(quote!(#pt ,));
                        field_names.push(quote!(#name ,));
                    }
                }
            }

            if world_field.is_none() {
                panic!("Commands must take in a `&mut World` parameter");
            }

            let field_frag = if fields.is_empty() {
                quote!( ; )
            }
            else {
                quote!( { #(#fields)* } )
            };

            let impl_command_frag = quote!(
                impl #generics bevy::ecs::system::Command for #cmd_name #generic_names {
                    fn apply(self, #world_field) {
                        let #cmd_name {#(#field_names)*} = self;
                        #block
                    }
                }
            );

            let commands_trait_frag = quote!(
                pub trait #trait_name {
                    #(#attrs)*
                    fn #method_name #generics (&mut self, #(#fields)*);
                }

                impl #trait_name for Commands<'_, '_> {
                    fn #method_name #generics (&mut self, #(#fields)*) {
                        self.add(#cmd_name {#(#field_names)*});
                    }
                }
            );

            let full = quote!(
                #(#attrs)*
                #vis
                #constness
                #asyncness
                #unsafety
                #abi
                struct
                #cmd_name
                #generics
                #field_frag
                #impl_command_frag
                #commands_trait_frag
            );

            println!("{full}");

            full
        }
        _ => panic!("unsupported item"),
    }.into()
}