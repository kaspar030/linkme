use syn::parse::{Parse, ParseStream, Result};
use syn::{LitInt, Path, Token};

pub enum Args {
    None,
    Path(Path),
    PathAndPos(Path, usize),
}

impl Parse for Args {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.is_empty() {
            Ok(Args::None)
        } else {
            let path: Path = input.parse()?;
            if input.is_empty() {
                Ok(Args::Path(path))
            } else {
                let _: Token![,] = input.parse()?;
                let pos_lit: LitInt = input.parse()?;
                let pos_val = pos_lit.base10_parse::<usize>()?;
                Ok(Args::PathAndPos(path, pos_val))
            }
        }
    }
}
