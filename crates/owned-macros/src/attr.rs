use syn::spanned::Spanned;

use crate::ctxt::Ctxt;

const TO_OWNED: &str = "to_owned";

/// Container attributes.
pub(crate) struct Container {
    // The name of the container.
    pub name: syn::Ident,
}

/// Parse container attributes.
pub(crate) fn container(
    cx: &Ctxt,
    ident: &syn::Ident,
    attrs: &[syn::Attribute],
) -> Result<Container, ()> {
    let mut container = Container {
        name: quote::format_ident!("Owned{}", ident),
    };

    for a in attrs {
        let result = a.parse_nested_meta(|parser| {
            if parser.path.is_ident("prefix") {
                let prefix: syn::Ident = parser.input.parse()?;
                container.name = quote::format_ident!("{prefix}{}", ident);
            } else {
                return Err(syn::Error::new(
                    parser.input.span(),
                    "Unsupported attribute",
                ));
            }

            Ok(())
        });

        if let Err(error) = result {
            cx.error(error);
        }
    }

    Ok(container)
}

#[derive(Default)]
pub(crate) enum FieldType {
    // Retain original field type.
    #[default]
    Original,
    // Replace with type.
    Type(syn::Type),
}

pub(crate) struct Field {
    pub(crate) is_set: bool,
    // Replace the type of the field.
    pub(crate) ty: FieldType,
    pub(crate) borrow: syn::Path,
    pub(crate) copy: bool,
    pub(crate) to_owned: syn::Path,
}

/// Parse container attributes.
pub(crate) fn field(cx: &Ctxt, attrs: &mut Vec<syn::Attribute>) -> Result<Field, ()> {
    let mut field = Field {
        is_set: false,
        ty: FieldType::default(),
        borrow: cx.borrow.clone(),
        copy: false,
        to_owned: cx.to_owned.clone(),
    };

    for a in attrs.iter() {
        if !a.path().is_ident(TO_OWNED) {
            continue;
        }

        field.is_set = true;

        let result = a.parse_nested_meta(|meta| {
            if meta.path.is_ident("ty") {
                meta.input.parse::<syn::Token![=]>()?;
                field.ty = FieldType::Type(meta.input.parse()?);
            } else if meta.path.is_ident("with") {
                meta.input.parse::<syn::Token![=]>()?;
                let path: syn::Path = meta.input.parse()?;

                field.to_owned = path.clone();
                field.to_owned.segments.push(syn::PathSegment {
                    ident: syn::Ident::new("to_owned", meta.path.span()),
                    arguments: syn::PathArguments::None,
                });

                field.borrow = path.clone();
                field.borrow.segments.push(syn::PathSegment {
                    ident: syn::Ident::new("borrow", meta.path.span()),
                    arguments: syn::PathArguments::None,
                });
            } else if meta.path.is_ident("copy") {
                field.copy = true;
            } else {
                return Err(syn::Error::new(meta.path.span(), "Unsupported attribute"));
            }

            Ok(())
        });

        if let Err(error) = result {
            cx.error(error);
        }
    }

    strip(attrs);
    Ok(field)
}

pub(crate) fn strip(attrs: &mut Vec<syn::Attribute>) {
    attrs.retain(|a| !a.path().is_ident(TO_OWNED));
}
