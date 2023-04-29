mod attr;
mod ctxt;
mod implement;

use ctxt::Ctxt;
use syn::parse::ParseStream;
use syn::spanned::Spanned;

#[proc_macro_attribute]
pub fn to_owned(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let parser = |input: ParseStream<'_>| input.call(syn::Attribute::parse_outer);

    let attr = syn::parse_macro_input!(attr with parser);
    let item = syn::parse_macro_input!(item as syn::Item);

    let cx = Ctxt::new(item.span());

    if let Ok(stream) = implement::implement(&cx, attr, item) {
        if !cx.has_errors() {
            return stream.into();
        }
    }

    cx.into_errors().into()
}
