use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
	parse_macro_input,
	Data,
	DeriveInput,
	Fields,
	Generics,
	Ident,
};

#[proc_macro_derive(Resource)]
pub fn derive_editable(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);
	expand_derive_resource(&input).into()
}

fn expand_derive_resource(input: &DeriveInput) -> TokenStream2 {
	match &input.data {
		Data::Struct(data) => {
			implement_struct_resource(&input.ident, &input.generics, &data.fields)
		}
		_ => unimplemented!("Enum not supported"),
	}
}

fn implement_struct_resource(ident: &Ident, generics: &Generics, _fields: &Fields) -> TokenStream2 {
	let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

	// let mut tokens = Vec::new();
	// match fields {
	// 	Fields::Named(fields) => {
	// 		for field in fields.named.iter() {
	// 			match &field.ident {
	// 				Some(ident) => {
	// 					tokens.push(quote! {
	// 						// newport_editor::Editable::edit(&mut self.#ident, stringify!(#ident), ui);
	// 					});
	// 				}
	// 				None => {}
	// 			}
	// 		}
	// 	}
	// 	_ => panic!("Named are only supported"),
	// }

	quote! {
		impl #impl_generics resources::Resource for #ident #ty_generics #where_clause {
			// fn edit(&mut self, name: &str, ui: &mut newport_editor::Ui) {
			// 	newport_editor::CollapsingHeader::new(name)
			// 		.default_open(true)
			// 		.show(ui, |ui|{
			// 			#(#tokens)*
			// 		});
			// }
		}
	}
}

#[proc_macro_derive(Inherit)]
pub fn derive_inherit(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);
	expand_derive_inherit(&input).into()
}

fn expand_derive_inherit(input: &DeriveInput) -> TokenStream2 {
	match &input.data {
		Data::Struct(data) => implement_struct_inherit(&input.ident, &input.generics, &data.fields),
		_ => unimplemented!("Enum not supported"),
	}
}

fn implement_struct_inherit(ident: &Ident, generics: &Generics, fields: &Fields) -> TokenStream2 {
	let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

	let ty = match fields {
		Fields::Named(fields) => {
			let field = fields.named.iter().find(|f| match &f.ident {
				Some(ident) => *ident == "base",
				None => false,
			});

			match field {
				Some(field) => field.ty.to_owned(),
				None => panic!("No field named base found"),
			}
		}
		_ => panic!("Named are only supported for now"),
	};

	quote! {
		impl #impl_generics std::ops::Deref for #ident #ty_generics #where_clause {
			type Target = #ty;
			fn deref(&self) -> &Self::Target {
				&self.base
			}
		}

		impl #impl_generics std::ops::DerefMut for #ident #ty_generics #where_clause {
			fn deref_mut(&mut self) -> &mut Self::Target {
				&mut self.base
			}
		}
	}
}
