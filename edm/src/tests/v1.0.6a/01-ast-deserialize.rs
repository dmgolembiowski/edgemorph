use test_case::test_case;
use pest::{Parser};
use pest_derive::Parser;

use quote::{quote};
use proc_macro2::TokenStream; 

#[test_case(
    /* gram_path  = */ "tests/v1.0.6a/pest/empty_module.pest", 
    /* ast_output = */ "tests/serialized/empty_module.txt",
    /* test_func_name  = */ "build_empty_module",
    /* test_func_anno  = */ "Schema"; 
    "Compiling an empty schema module: `tests/serialized/empty_module.esdl`")]
fn test_deserialization(
            gram_path: &str, 
            ast_output: &str,
            test_func_name: &str,
            test_func_anno: &str) 
{
    // Prepare a singleton parser from the Pest file
    let pest_loc = quote! { gram_path.to_owned(); };
    let interpolated = quote! {
        #[derive(Parser)]
        #[grammar = #pest_loc]
        pub struct Deserializer;
    };
    TokenStream::from(interpolated);
    
    // Build the deserialized AST struct corresponding
    // to the function name and type annotation
    let func = quote! { test_func_name.to_owned();  };
    let func_arg = quote! { ast_output.borrow(); };
    let anno = quote! { test_func_anno.to_owned(); };

    
    let expanded = quote! {
        match #func.to_string() {
            "build_empty_module" => {
                //let actual = common::deserialize(&Deserializer, &ast_output);
                let actual = Deserializer::parse(Rule::ident_list, ast_output);
                let theoretical: #anno = #func(#func_arg);
                let res = assert_eq!(actual, theoretical);
                match res {
                    false => { panic!("Failed to compile."); },
                    _ => {}
                }
            },
        }
    };

    TokenStream::from(expanded);
    
}

// This is merely a dummy to verify that the dynamic dispatch
// for the test harness works properly.
#[allow(dead_code)]
pub struct Schema {
    declarations: Vec<i32>
}

#[allow(dead_code)]
fn build_empty_module(ast_path: &str) -> Schema {
    Schema { declarations: vec![1] }
}
#[allow(dead_code)]
fn main() {}
