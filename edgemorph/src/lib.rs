pub mod datastructures;
use datastructures::{TypeBuilder, Type};

pub fn new_scalar_type(ident: &str) -> Type {
    TypeBuilder::default()
        .ident(ident)
        .scalar(true)
        .build()
        .unwrap()
}

pub fn _new_object_type(ident: &str) -> Type {
    TypeBuilder::default()
        .ident(ident)
        .scalar(false)
        .build()
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
 
    #[test]
    fn gen_scalar() {
        let identity = "Double";
        let scalar = new_scalar_type(&identity);
        dbg!(&scalar);
        assert_eq!(scalar.scalar, true);
    }
}
