use proc_macro2::Ident;
use quote::ToTokens;

use syn::spanned::Spanned;

use syn::{Error, Expr, ExprLit, Lit, Path};

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
