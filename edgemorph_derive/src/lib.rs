use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{parse_macro_input, parse_quote, Data, DeriveInput, Fields, GenericParam, Generics, Index};
use std::cell::RefCell;
use std::rc::Weak;
use std::boxed::Box;

#[proc_macro_derive(Containerize)]
pub fn derive_containerize(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let generics = add_trait_bounds(input.generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    
    // Generate expressions to apply new builder patterns
    let tbv = make_box_vec(&input.data);
    let twr = make_weak_refcell(&input.data);
    let tb  = make_box(&input.data);

    let expanded = quote! {
        // Expose containerization trait to the datastructures builder
        impl #impl_generics edgemorph::Containerize for #name #ty_generics #where_clause {
            fn to_box_vec(self) -> Box<Vec<Self>> {
                #tbv
            }
            fn to_weak_refcell(self) -> RefCell<Weak<Self>> {
                #twr
            } 
            fn to_box(self) -> Box<Self> {
                #tb
            }
        }
    };
    // Hand the output tokens back to the compiler.
    proc_macro::TokenStream::from(expanded)
}

// Add a bound `T: Containerize` to every type parameter T.
fn add_trait_bounds(mut generics: Generics) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param.bounds.push(parse_quote!(edgemorph::Containerize));
        }
    }
    generics
}

// Generate an expression to return Box<Vec<Self>>
fn make_box_vec(data: &Data) -> TokenStream {
    match *data {
        Data::Struct(ref data) => {
            match data.fields {
                Fields::Named(ref fields) => {
                    let recurse = fields.named.iter().map(|f| {
                        let name = &f.ident;
                        quote_spanned! {f.span()=>
                            edgemorph::Containerize::to_box_vec(&self.#name)
                        }
                    });
                    quote! {
                        #(#recurse)*
                    }
                }
                _ => unimplemented!()
            }
        }
        Data::Enum(_) | Data::Union(_) => unimplemented!(),
    }
}

// Generate an expression to return Box<Vec<Self>>
fn make_weak_refcell(data: &Data) -> TokenStream {
    match *data {
        Data::Struct(ref data) => {
            match data.fields {
                Fields::Named(ref fields) => {
                    let recurse = fields.named.iter().map(|f| {
                        let name = &f.ident;
                        quote_spanned! {f.span()=>
                            edgemorph::Containerize::to_weak_refcell(&self.#name)
                        }
                    });
                    quote! {
                        #(#recurse)*
                    }
                }
                _ => unimplemented!()
            }
        }
        Data::Enum(_) | Data::Union(_) => unimplemented!(),
    }
}


// Generate an expression to return Box<Vec<Self>>
fn make_box(data: &Data) -> TokenStream {
    match *data {
        Data::Struct(ref data) => {
            match data.fields {
                Fields::Named(ref fields) => {
                    let recurse = fields.named.iter().map(|f| {
                        let name = &f.ident;
                        quote_spanned! {f.span()=>
                            edgemorph::Containerize::to_box(&self.#name)
                        }
                    });
                    quote! {
                        #(#recurse)*
                    }
                }
                _ => unimplemented!()
            }
        }
        Data::Enum(_) | Data::Union(_) => unimplemented!(),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
