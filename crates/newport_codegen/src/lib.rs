use proc_macro::TokenStream;
use syn::{ parse_macro_input, DeriveInput, Data };
use quote::quote;

#[proc_macro_derive(Editable)]
pub fn derive_editable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let ident = input.ident;
    match input.data {
        Data::Struct(data) => {
            let tokens = quote! {
                impl Editable for #ident {
                    fn show(name: &str, ui: &mut Ui) {
                        
                    }
                }
            };
            tokens.into()
        },
        _ => unimplemented!()
    }
}