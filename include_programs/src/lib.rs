use std::{env, fs, path::PathBuf};

use proc_macro::TokenStream;
use quote::quote;

#[proc_macro]
pub fn include_programs(_input: TokenStream) -> TokenStream {
    let current_dir = env::current_dir().unwrap();
    let program_dir = current_dir.join(PathBuf::from("kernel/programs"));

    let program_files = fs::read_dir(program_dir)
        .expect("No programs directory is present")
        .flatten()
        .filter_map(|file| {
            let path = file.path();

            if path.is_file() {
                let contents = fs::read(path).unwrap();
                return Some(contents);
            }

            None
        })
        .collect::<Vec<_>>();

    let tokens: Vec<_> = program_files
        .iter()
        .map(|inner_vec| {
            quote! {
            &[#(#inner_vec), *]
            }
        })
        .collect();

    let expanded = quote! {
        &[
            #(
                #tokens
            ),*
        ]
    };

    expanded.into()
}
