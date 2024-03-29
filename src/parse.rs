use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Comma;
use syn::{
    parse_quote, Attribute, Error, Expr, ExprLit, FnArg, GenericArgument, Lit, Meta, MetaNameValue,
    Pat, Path, PathArguments, ReturnType, Type,
};

pub struct MacroArgs {
    pub no_trait: bool,
    pub no_world: bool,
    pub name: Ident,
    pub struct_name: Option<Ident>,
    pub trait_name: Option<Ident>,
    pub ecs_root: Option<Path>,
}

/// parse macro args
pub fn macro_args(args: &Punctuated<Meta, Comma>, mut name: Ident) -> Result<MacroArgs, Error> {
    // arguments
    let mut no_trait = false;
    let mut no_world = false;
    let mut struct_name = None;
    let mut trait_name = None;
    let mut ecs_root = None;

    // parse macro arguments
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

    Ok(MacroArgs {
        no_trait,
        no_world,
        name,
        struct_name,
        trait_name,
        ecs_root,
    })
}

pub struct SysArgs {
    pub entity: Option<TokenStream>,
    pub fields: Vec<TokenStream>,
    pub def_field_names: Vec<TokenStream>,
    pub impl_field_names: Vec<TokenStream>,
    pub args: SystemArgs,
}

pub enum SystemArgs {
    /// Exclusive commands always have one SystemParam: &mut World
    /// All other params are inherently inputs
    Exclusive { world: TokenStream },
    /// System commands have multiple SystemParams
    /// All inputs must be packed into the `In<T>` struct
    /// eg. `In((entity, n)): In<(Entity, usize)>, mut query: Query<&mut TestUsize>`
    System {
        entity_name: Option<TokenStream>,
        systems_in: Vec<TokenStream>,
    },
}

/// parse command args
pub fn fn_args(inputs: &Punctuated<FnArg, Comma>, entity_command: bool) -> Result<SysArgs, Error> {
    let mut exclusive_fields = Vec::<TokenStream>::new();
    let mut exclusive_def_field_names = Vec::<TokenStream>::new();
    let mut exclusive_impl_field_names = Vec::<TokenStream>::new();
    let mut system_fields = Vec::<TokenStream>::new();
    let mut system_def_field_names = Vec::<TokenStream>::new();
    let mut system_impl_field_names = Vec::<TokenStream>::new();
    let mut systems_in = Vec::<TokenStream>::new();
    let mut entity_name = None;
    let mut world_field = None;
    let mut entity_field = None;

    for input in inputs {
        match input {
            // `self` types smell of methods
            FnArg::Receiver(inner) => {
                return Err(Error::new(inner.span(), "Commands cannot be methods"))
            }
            FnArg::Typed(pt) => {
                let name = pt.pat.clone();
                // handle `&World`, `Entity`, and `In<>` types specially
                // builds a list of all types in the various parts necessary for generation
                match pt.ty.as_ref() {
                    Type::Reference(tr) => {
                        if tr.elem.to_token_stream().to_string() == "World" {
                            world_field = Some(quote!(#pt));
                            continue;
                        }
                    }
                    Type::Path(path) => {
                        if let Some(seg) = path.path.segments.last() {
                            let ident = &seg.ident;
                            if entity_command && ident == "Entity" {
                                entity_field = Some(quote!(#pt));
                                continue;
                            } else if ident == "In" {
                                // in this case we need to additionally parse the parameter name which may expand into more through destructuring
                                // normally destructuring is not allowed in commands macros, but it's needed in this style to support more than one input arg
                                // todo: support destructuring in regular command macros because I hate myself?

                                let mut names = Vec::new();
                                let Pat::TupleStruct(pat) = *pt.pat.clone() else {
                                    return Err(Error::new(pt.span(), "Unsupported input type"));
                                };
                                // Parse inner names for elements of In<elem> or In<(elems,)>
                                for pat in pat.elems {
                                    match pat {
                                        Pat::Ident(pat) => names.push(pat),
                                        Pat::Tuple(pt) => {
                                            for pat in pt.elems {
                                                let Pat::Ident(pat) = pat else {
                                                    return Err(Error::new(
                                                        pat.span(),
                                                        "Invalid path",
                                                    ));
                                                };
                                                names.push(pat);
                                            }
                                        }
                                        _ => return Err(Error::new(pat.span(), "Unknown input")),
                                    }
                                }

                                // Parse inner types of In<ty> or In<(tys,)>
                                let mut args: Vec<TokenStream> = Vec::new();
                                match &seg.arguments {
                                    PathArguments::AngleBracketed(inner) => {
                                        for arg in &inner.args {
                                            let GenericArgument::Type(ty) = arg else {
                                                return Err(Error::new(
                                                    arg.span(),
                                                    "Unknown argument type",
                                                ));
                                            };

                                            match ty {
                                                Type::Tuple(tt) => {
                                                    for ty in &tt.elems {
                                                        args.push(ty.to_token_stream());
                                                    }
                                                }
                                                Type::Path(tp) => args.push(tp.to_token_stream()),
                                                _ => {
                                                    return Err(Error::new(
                                                        arg.span(),
                                                        "Unsupported argument type",
                                                    ))
                                                }
                                            }
                                        }
                                    }
                                    _ => {
                                        return Err(Error::new(
                                            path.span(),
                                            "Unsupported use of `In`",
                                        ));
                                    }
                                };

                                // 1:1 name:type mapping
                                if names.len() == args.len() {
                                    for (pat, arg) in names.into_iter().zip(args) {
                                        let name = &pat.ident;
                                        if entity_command && arg.to_string() == "Entity" {
                                            entity_name = Some(quote!(#name));
                                            entity_field = Some(quote!(#name: #arg));
                                            systems_in.push(quote!(#name));
                                            continue;
                                        }
                                        system_fields.push(quote!(#name: #arg));
                                        system_def_field_names.push(quote!(#name));
                                        system_impl_field_names.push(quote!(#pat));
                                        systems_in.push(quote!(#name));
                                    }
                                }
                                // 1:many name:type mapping
                                else if names.len() == 1 && !args.is_empty() {
                                    let pat = names.first().unwrap();
                                    let name = &pat.ident;
                                    system_fields.push(quote!(#name: (#(#args,)*)));
                                    system_def_field_names.push(quote!(#name));
                                    system_impl_field_names.push(quote!(#pat));
                                    systems_in.push(quote!(#name));
                                } else {
                                    return Err(Error::new(
                                        path.span(),
                                        "Imbalanced names and types",
                                    ));
                                }

                                continue;
                            }
                        }
                    }
                    _ => (),
                }

                // these fields are not `&mut World`, `Entity`, nor `In`
                // they only matter for exclusive systems, for normal systems these are the system parameters included by the root system

                let Pat::Ident(pat) = *name.clone() else {
                    return Err(Error::new(name.span(), "Invalid path"));
                };
                let name = &pat.ident;
                let ty = &pt.ty;

                exclusive_fields.push(quote!(#name: #ty));
                exclusive_def_field_names.push(quote!(#name));
                exclusive_impl_field_names.push(quote!(#pat));
            }
        }
    }

    // figure these out late since some parts have different meanings depending on whether this is an exclusive or normal system
    let (fields, def_field_names, impl_field_names) = if world_field.is_some() {
        (
            exclusive_fields,
            exclusive_def_field_names,
            exclusive_impl_field_names,
        )
    } else {
        (
            system_fields,
            system_def_field_names,
            system_impl_field_names,
        )
    };

    let args = match world_field {
        Some(world) => SystemArgs::Exclusive { world },
        None => SystemArgs::System {
            entity_name,
            systems_in,
        },
    };

    Ok(SysArgs {
        entity: entity_field,
        fields,
        def_field_names,
        impl_field_names,
        args,
    })
}

/// parse the return type of a function & check whether it's our special marker or not
pub fn return_type(output: &ReturnType) -> Result<bool, Error> {
    let ret = match &output {
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

    Ok(ret)
}

/// separate out doc comments for our trait method
pub fn docs(attrs: &[Attribute]) -> TokenStream {
    let mut docs = Vec::new();
    for attr in attrs {
        if let Meta::NameValue(meta) = &attr.meta {
            if meta.path.is_ident("doc") {
                docs.push(attr);
            }
        }
    }
    quote!(#(#docs)*)
}

pub trait ExprExt {
    fn try_to_path(&self) -> Result<Path, Error>;
    fn try_to_ident(&self) -> Result<Ident, Error>;
}

impl ExprExt for Expr {
    fn try_to_path(&self) -> Result<Path, Error> {
        let path = match &self {
            Expr::Lit(ExprLit {
                lit: Lit::Str(lit), ..
            }) => lit.parse_with(Path::parse_mod_style)?,
            Expr::Path(path) => path.path.clone(),
            value => {
                return Err(Error::new(
                    value.span(),
                    format!("invalid path: `{}`", value.to_token_stream()),
                ))
            }
        };
        Ok(path)
    }

    fn try_to_ident(&self) -> Result<Ident, Error> {
        let ident = match &self {
            Expr::Lit(ExprLit {
                lit: Lit::Str(lit), ..
            }) => lit.parse()?,
            Expr::Path(path) => {
                if path.path.segments.is_empty() {
                    return Err(Error::new(path.span(), "Name must exist"));
                }
                if path.path.segments.len() > 1 {
                    return Err(Error::new(path.span(), "Name must be an ident, found path"));
                }
                path.path.clone().segments.pop().unwrap().into_value().ident
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
}
