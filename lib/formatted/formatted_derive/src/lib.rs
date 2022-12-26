extern crate proc_macro;
extern crate quote;

use quote::{quote, ToTokens};
use synstructure::{decl_derive, Structure, unpretty_print};

decl_derive!([Formatted] => derive_formatted);
fn derive_formatted(structure: Structure) -> proc_macro2::TokenStream {
    let readable = structure.each(|bi| {
        let field = bi.ast();
        println!("AMOGUS");
        field.attrs.iter().for_each(|a| {
            println!("{:?}", unpretty_print(a.tokens.into_token_stream()));
        });

        println!("{:?} {:?}", field.attrs, field.ident);
        quote! {
            walk(#bi)
        }
    });

    structure.gen_impl(quote! {
        extern crate formatted;
        use io::{Read, Seek, Write};

        gen impl formatted::Formatted for @Self {
            fn load_readable<T: Read>(stream: &mut T) -> Result<Self> {

            }

            fn load_readable_seekable<T: Read + Seek>(stream: &mut T) -> Result<Self> {

            }

            fn save_writable<T: Write>(&self, stream: &mut T) {

            }

            fn save_writable_seekable<T: Write + Seek>(&self, stream: &mut T) {

            }
        }
    })
}