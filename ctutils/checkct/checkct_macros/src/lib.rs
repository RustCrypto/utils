use proc_macro::TokenStream;
use proc_macro2::{Literal, Span};
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, Error, Ident, ItemFn, LitStr, Result, Token,
};

struct AttributeArgs {
    descriptor_link_section: Option<String>,
}

impl Parse for AttributeArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.is_empty() {
            return Ok(Self {
                descriptor_link_section: None,
            });
        }

        let arg_name: Ident = input.parse()?;
        if arg_name != "descriptor_link_section" {
            return Err(Error::new(
                arg_name.span(),
                "expected `descriptor_link_section`",
            ));
        }
        input.parse::<Token![=]>()?;
        let descriptor_link_section: LitStr = input.parse()?;

        Ok(Self {
            descriptor_link_section: Some(descriptor_link_section.value()),
        })
    }
}

#[proc_macro_attribute]
pub fn checkct(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    // By default the descriptor will be stored in the .note.checkct section
    let descriptor_link_section = Literal::string(
        args.descriptor_link_section
            .as_deref()
            .unwrap_or(".note.checkct"),
    );

    let item_fn = parse_macro_input!(input as ItemFn);
    let name = &item_fn.sig.ident;

    let entrypoint_descriptor_name = Ident::new(
        &format!("__checkct_entrypoint_descriptor__{name}"),
        Span::call_site(),
    );
    let entrypoint_descriptor_type = Ident::new(
        &format!("__CheckctEntrypointDescriptor__{name}"),
        Span::call_site(),
    );

    quote! {
        // The descriptor is laid out as an ELF note (which binsec parses), since it is stored
        // in a note section by default. It ends with the address of the entrypoint.
        #[doc(hidden)]
        #[allow(non_camel_case_types, dead_code)]
        #[repr(C, packed(4))]
        pub struct #entrypoint_descriptor_type {
            namesz: u32,
            descsz: u32,
            kind: u32,
            name: [u8; 8],
            entrypoint: fn(),
        }

        // This variable is used to prevent the compiler from removing the code of the entrypoint
        // function if it is unused. Moreover this variable name has a special format which allows
        // cargo-checkct to find it.
        // Make sure that the descriptor link section is kept by the linker.
        #[unsafe(link_section = #descriptor_link_section)]
        #[used]
        pub static #entrypoint_descriptor_name: #entrypoint_descriptor_type =
            #entrypoint_descriptor_type {
                namesz: 8,
                descsz: ::core::mem::size_of::<fn()>() as u32,
                kind: 1,
                name: *b"checkct\0",
                entrypoint: #name,
            };

        #item_fn
    }
    .into()
}
