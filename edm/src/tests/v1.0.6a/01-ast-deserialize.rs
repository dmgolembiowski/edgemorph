use test_case::test_case;
use pest::Parser;
use pest_derive::Parser;
use quote::{quote};
use proc_macro2::TokenStream;

mod helper; 

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
        //let actual = common::deserialize(&Deserializer, &ast_output);
        let actual = Deserializer::parse(Rule::ident_list, *ast_output);
        let theoretical: #anno = helper::dyn_call!(#func(#func_arg));
        assert_eq!(actual, theoretical);
    };

    /* "[Parse] the function name (either a string literal or 
       an ident for an &str variable) and the args in the function 
       call syntax. 
       [Then ... take] the function name and passes it to [helper::get_sym], 
       which is just a function for reading the executable from the first arg 
       passed and getting a pointer to a dynsym by name. 
       It then transmutes it to a function pointer so we can call it." -Jam
       */
    TokenStream::from(expanded);
    
}

// This is merely a dummy to verify that the dynamic dispatch
// for the test harness works properly.
pub struct Schema {
    declarations: Vec<i32>
}

#[no_mangle]
pub extern "Rust" fn build_empty_module(ast_path: &str) -> Schema {
    Schema { declarations: vec![1] }
}

fn main() {}
