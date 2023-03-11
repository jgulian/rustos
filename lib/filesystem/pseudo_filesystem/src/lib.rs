
#[proc_macro_attribute]
pub fn read(attr: TokenStream, item: TokenStream) -> TokenStream {
    println!("attr: \"{}\"", attr.to_string());
    println!("item: \"{}\"", item.to_string());
    item
}

#[proc_macro_attribute]
pub fn write(attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

macro_rules! files {
    ($file:ident) => {

    };
    ($($files:ident),+ $file:ident) => {
        files!(file)
        files!(files)
    }
}