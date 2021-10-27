#![recursion_limit = "128"]

use proc_macro::TokenStream;

extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

#[proc_macro_derive(VertexAttribPointers, attributes(location))]
pub fn vertex_attrib_pointers_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse(input).unwrap();
    generate_impl(&ast)
}

fn generate_impl(ast: &syn::DeriveInput) -> TokenStream {
    let ident = &ast.ident;
    let generics = &ast.generics;
    let where_clause = &ast.generics.where_clause;
    let fields_vertex_attrib_pointer = generate_vertex_attrib_pointer_calls(&ast.data);

    let gen = quote! {
        impl #ident #generics #where_clause {
            #[allow(unused_variables)]
            pub fn vertex_attrib_pointers() {
                let stride = ::std::mem::size_of::<Self>();
                let offset = 0;

                #(#fields_vertex_attrib_pointer)*
            }
        }
    };

    gen.into()
}

fn generate_vertex_attrib_pointer_calls(body: &syn::Data) -> Vec<Box<dyn quote::ToTokens>> {
    match body {
        &syn::Data::Union(_) => panic!("VertexAttribPointers can not be implemented for unions"),
        &syn::Data::Enum(_) => panic!("VertexAttribPointers can not be implemented for enums"),
        // &syn::Data::Struct(syn::VariantData::Unit) => {
        //     panic!("VertexAttribPointers can not be implemented for Unit structs")
        // }
        // &syn::Data::Struct(syn::VariantData::Tuple(_)) => {
        //     panic!("VertexAttribPointers can not be implemented for Tuple structs")
        // }
        // &syn::Data::Struct(syn::VariantData::Struct(ref s)) => s
        &syn::Data::Struct(ref data_struct) => data_struct
            .fields
            .iter()
            .map(generate_struct_field_vertex_attrib_pointer_call)
            .collect(),
    }
}

fn generate_struct_field_vertex_attrib_pointer_call(
    field: &syn::Field,
) -> Box<dyn quote::ToTokens> {
    let field_name = match field.ident {
        Some(ref i) => format!("{}", i),
        None => String::from(""),
    };
    let location_attr = field
        .attrs
        .iter()
        .filter_map(|attr| attr.parse_meta().ok())
        .filter(|meta| match meta {
            syn::Meta::NameValue(syn::MetaNameValue { ref path, .. }) => path
                .get_ident()
                .map(|ident| ident == "location")
                .unwrap_or(false),
            _ => false,
        })
        .take(1)
        .next()
        .unwrap_or_else(|| panic!("Field {} is missing #[location = ?] attribute", field_name));

    let location_value = match location_attr {
        syn::Meta::NameValue(syn::MetaNameValue { ref lit, .. }) => lit,
        _ => panic!(
            "Field {} location attribute value must be an integer literal",
            field_name
        ),
    };

    let field_ty = &field.ty;
    let gen = quote! {
        let location = #location_value;
        unsafe {
            #field_ty::vertex_attrib_pointer(stride, location, offset);
        }
        let offset = offset + ::std::mem::size_of::<#field_ty>();
    };

    Box::new(gen)
}
