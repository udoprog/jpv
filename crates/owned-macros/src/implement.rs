use proc_macro2::{Span, TokenStream};
use quote::{quote_spanned, ToTokens};
use syn::spanned::Spanned;

use crate::attr;
use crate::ctxt::Ctxt;

const NAME: &str = "#[owned]";

enum Call<'a> {
    Path(syn::token::And, &'a syn::Path),
    Copy,
}

impl Call<'_> {
    fn ident(&self, span: Span, ident: &syn::Ident) -> TokenStream {
        match self {
            Call::Path(bor, path) => quote_spanned!(span => #path(#bor self.#ident)),
            Call::Copy => quote_spanned!(span => self.#ident),
        }
    }

    fn index(&self, span: Span, index: usize) -> TokenStream {
        match self {
            Call::Path(bor, path) => quote_spanned!(span => #path(#bor self.#index)),
            Call::Copy => quote_spanned!(span => self.#index),
        }
    }
}

pub(crate) fn implement(
    cx: &Ctxt,
    attrs: Vec<syn::Attribute>,
    mut item: syn::Item,
) -> Result<TokenStream, ()> {
    let mut output = item.clone();
    let mut to_owned_entries = Vec::new();
    let mut borrow_entries = Vec::new();

    match (&mut output, &mut item) {
        (syn::Item::Struct(st), syn::Item::Struct(b_st)) => {
            let container = attr::container(cx, &st.ident, &attrs);
            let container = container?;
            st.ident = container.name;

            strip_generics(&mut st.generics);

            for (index, (field, b_field)) in
                st.fields.iter_mut().zip(b_st.fields.iter_mut()).enumerate()
            {
                let attr = attr::field(cx, &mut field.attrs);
                let attr = attr?;

                attr::strip(&mut b_field.attrs);

                if let Some(meta) = attr.borrowed_meta {
                    b_field.attrs.push(syn::Attribute {
                        pound_token: syn::token::Pound::default(),
                        style: syn::AttrStyle::Outer,
                        bracket_token: syn::token::Bracket::default(),
                        meta,
                    });
                }

                if let attr::FieldType::Type(ty) = attr.ty {
                    field.ty = ty;
                }

                let and = <syn::Token![&]>::default();

                let (to_owned, borrow) = if attr.copy {
                    (Call::Copy, Call::Copy)
                } else if attr.is_set {
                    (
                        Call::Path(and, &attr.to_owned),
                        Call::Path(and, &attr.borrow),
                    )
                } else {
                    let clone = &cx.clone;
                    (Call::Path(and, clone), Call::Path(and, clone))
                };

                match &field.ident {
                    Some(ident) => {
                        let f = to_owned.ident(ident.span(), ident);
                        to_owned_entries.push(quote_spanned!(ident.span() => #ident: #f));
                        let f = borrow.ident(ident.span(), ident);
                        borrow_entries.push(quote_spanned!(ident.span() => #ident: #f));
                    }
                    None => {
                        let f = to_owned.index(field.span(), index);
                        to_owned_entries.push(quote_spanned!(field.span() => #index: #f));
                        let f = to_owned.index(field.span(), index);
                        borrow_entries.push(quote_spanned!(field.span() => #index: #f));
                    }
                }
            }
        }
        (syn::Item::Enum(en), syn::Item::Enum(b_en)) => {
            let container = attr::container(cx, &en.ident, &attrs);
            let container = container?;
            en.ident = container.name;

            strip_generics(&mut en.generics);

            for (variant, b_variant) in en.variants.iter_mut().zip(b_en.variants.iter_mut()) {
                for (field, b_field) in variant.fields.iter_mut().zip(b_variant.fields.iter_mut()) {
                    let attr = attr::field(cx, &mut field.attrs);
                    let attr = attr?;

                    attr::strip(&mut b_field.attrs);

                    if let attr::FieldType::Type(ty) = attr.ty {
                        field.ty = ty;
                    }
                }
            }
        }
        (_, item) => {
            cx.span_error(
                item.span(),
                format_args!("{} is only supported on structs and enum", NAME),
            );
            return Err(());
        }
    };

    let (owned_ident, owned_generics) = match &output {
        syn::Item::Struct(st) => (&st.ident, &st.generics),
        _ => return Err(()),
    };

    let (borrow_ident, borrow_generics) = match &item {
        syn::Item::Struct(st) => (&st.ident, &st.generics),
        syn::Item::Enum(en) => (&en.ident, &en.generics),
        _ => {
            return Err(());
        }
    };

    let (_, to_owned_type_generics, _) = owned_generics.split_for_impl();

    let to_owned = {
        let (impl_generics, type_generics, where_generics) = borrow_generics.split_for_impl();
        let to_owned = &cx.owned_to_owned;

        quote_spanned! {
            item.span() =>
            #[automatically_derived]
            impl #impl_generics #to_owned for #borrow_ident #type_generics #where_generics {
                type Owned = #owned_ident #to_owned_type_generics;

                #[inline]
                fn to_owned(&self) -> Self::Owned {
                    #owned_ident {
                        #(#to_owned_entries,)*
                    }
                }
            }
        }
    };

    let (_, borrow_return_type_generics, _) = borrow_generics.split_for_impl();

    let borrow = {
        let (impl_generics, type_generics, where_generics) = owned_generics.split_for_impl();
        let owned_borrow = &cx.owned_borrow;

        quote_spanned! {
            item.span() =>
            #[automatically_derived]
            impl #impl_generics #owned_borrow for #owned_ident #type_generics #where_generics {
                type Target<'a> = #borrow_ident #borrow_return_type_generics;

                fn borrow(&self) -> Self::Target<'_> {
                    #borrow_ident {
                        #(#borrow_entries,)*
                    }
                }
            }
        }
    };

    let mut stream = TokenStream::new();
    item.to_tokens(&mut stream);
    output.to_tokens(&mut stream);
    to_owned.to_tokens(&mut stream);
    borrow.to_tokens(&mut stream);
    Ok(stream)
}

fn strip_generics(generics: &mut syn::Generics) {
    let mut params = generics.params.clone();
    params.clear();

    for p in &generics.params {
        if !matches!(p, syn::GenericParam::Lifetime(..)) {
            params.push(p.clone());
        }
    }

    generics.params = params;
}
