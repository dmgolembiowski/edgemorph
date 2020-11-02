use test_case::test_case;
// use pest::iterators::Pair;
// use pest::prec_climber::{Assoc, Operator, PrecClimber};
use pest::Parser;
use pest_derive::Parser;

macro_rules! dyn_parser { 
    ($path:expr) => {  
        #[derive(Parser)]
        #[grammar = $path]
        pub struct Deserializer;

    }
}


#[test_case(
    /* gram_path = */ "../v1.0.6a/pest/empty_module.pest", 
    /* ast_output= */ "../serialized/empty_module.txt"; "empty_module.esdl")]
fn test_deserialization(gram_path: impl AsRef<str>, ast_output: &str) {
    dyn_parser!(gram_path);
    assert!(true);
}

fn main() {}
