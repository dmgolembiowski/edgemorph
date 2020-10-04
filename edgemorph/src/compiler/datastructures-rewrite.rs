use derive_builder::Builder;
use std::rc::Weak;
use std::cell::RefCell;
use std::boxed::Box;

#[derive(Debug, Clone, Default)]
pub struct Module {}

#[derive(Builder, Debug, Clone)]
#[builder(build_fn(validate = "Self::validate"))]
pub struct Type<'a, 'b, 'c> {

    #[builder(setter(into))]
    pub ident:       String,

    #[builder(setter(into), default = "false")]
    pub abs:         bool,
    
    #[builder(setter(into), default = "false")]
    pub scalar:      bool,
    
    #[builder(setter(into, strip_option), default)]
    pub extends:     Option<&'a [RefCell<Weak<SuperType<'b, 'c, 'c>>>]>,

    #[builder(setter(into, strip_option), default)]
    pub properties:  Option<Box<Vec<Property<'a>>>>,

    #[builder(setter(into, strip_option), default)]
    pub annotations: Option<Box<Vec<Annotation>>>,
    
    #[builder(setter(into, strip_option), default)]
    pub links:       Option<&'a [RefCell<Weak<Link<'a>>>]>,
    
    #[builder(setter(into, strip_option), default)]
    pub constraints: Option<Box<Vec<Constraint>>>,
    
    #[builder(setter(into, strip_option), default)]
    pub indices:     Option<Box<Index>>

}

impl<'a, 'b, 'c> TypeBuilder<'a, 'b, 'c> {
    /// Verify that `self.ident` is not an empty `String`
    fn validate(&self) -> Result<(), String> {
        if self.ident.as_ref().unwrap().is_empty() {
            Err("`Type.ident` must not be an empty String.".to_string())
        } else {
            Ok(())
        }
    }
}

#[derive(Builder, Clone, Debug, Default, Eq, PartialEq)]
#[builder(build_fn(validate = "Self::validate"))]
pub struct Annotation {
    #[builder(setter(into))]
    pub ident: String,
    
    #[builder(setter(into))]
    pub value: String
}

impl AnnotationBuilder {
    /// Verify that `self.ident` is not an empty `String`
    fn validate(&self) -> Result<(), String> {
        if self.ident.as_ref().unwrap().is_empty() {
            Err("`Type.ident` must not be an empty String.".to_string())
        } else {
            Ok(())
        }
    }
}

pub type SuperType<'a, 'b, 'c> = Type<'a, 'b, 'c>;
pub type ArgSpec = Annotation;

/// As I understand it, EdgeQL uses aliases as constructs
///  like sub-queries to cut down on boilerplate within a 
///  module's schema.
///
/// For example:
///  As DDL: `CREATE ALIAS Superusers := (SELECT User FILTER User.groups.name = "Superusers");
///  As SDL: ```
///          alias Superuser := User {
///              # need to double-check this 
///              groups: {
///                  name = "Superusers" 
///              }
///          }```
///
#[derive(Builder, Clone, Debug)]
#[builder(build_fn(validate = "Self::validate"))]
pub struct Alias {
    
    #[builder(setter(into))]
    pub ident: String,

    #[builder]
    pub alias_expr: AliasExpr,
}

impl AliasBuilder {
    /// Verify that `self.ident` is not an empty `String`
    fn validate(&self) -> Result<(), String> {
        if self.ident.as_ref().unwrap().is_empty() {
            Err("`AliasBuilder.ident` must not be an empty String.".to_string())
        } else {
            Ok(())
        }
    }   
}

#[derive(Builder, Debug, Clone)]
pub struct AliasExpr { 
    
    #[builder(setter(into, strip_option), default)]
    pub ddl: Option<String>,
    
    #[builder(setter(into, strip_option), default)]
    pub sdl: Option<String>

    // ToDo: Make traits for better building patterns + data-structural integrity
}

#[derive(Builder, Clone, Debug)]
pub struct Property<'a> { 

    #[builder(setter(into))]
    pub ident:         String,

    #[builder(default = "false")]
    pub overloaded: bool,

    #[builder(default = "false")]
    pub abs:        bool,

    #[builder(default = "false")]
    pub readonly:   bool,

    #[builder(default = "false")]
    pub required:   bool,

    #[builder(default = "false")]
    pub multi:      bool,

    // Seems like this should be a Weak<PropertyKind>
    // since we need to track ref counts without 
    // introducing memory leaks; if so, then need to
    // make a builder-setter that does:
    /*
    ```rust
        use std::rc::{Rc, Weak};
        // Then inside some builder function with a &mut self arg,
        // we also pass an argument 
        // resembling: `propkind: &'a PropertyKind` ... then inside
        // that function's scope we do:

        let pk: Weak<PropertyKind> = Rc::downgrade(propkind);
        *self.kind = pk;
    ```
    */
    // ToDo: update edgemorph_derive to handle simple Weak builder
    pub kind:          &'a PropertyKind, // -> Weak<PropertyKind> 

    // ToDo: Seems like this should be `Option<Rc<_>>` and
    // if it's given to us by edm as `some_default: Option<&'a Rc<Expression>>` 
    // (since mutability isn't needed). The reasoning is that each `default` then its edgemorph's job to make 
    // (comparatively cheaper) 8-byte copies of a  footprint and clone a cheap ref to the fat pointer
    // as:
    // ```rust
    //     match *some_default {
    //         Some(propkind) => { 
    //              *self.kind = Some(Rc::clone(propkind)) },
    //         None           => { *self.kind = None }
    //     }
    // ```

    //`to_owned()` (since the Expression is a singleton
    // Expression on t)
    #[builder(setter(into, strip_option))]
    pub default:       Option<&'a Expression>,

    #[builder(setter(into, strip_option))]
    pub constraints:   Option<Box<Vec<Constraint>>>,

    #[builder(setter(into, strip_option))]
    pub extends:       Option<&'a [RefCell<Weak<Property<'a>>>]>,

    #[builder(setter(into, strip_option))]
    pub module:        Option<RefCell<Weak<Module>>>
}

#[allow(dead_code)]
#[allow(non_snake_case)]
#[derive(Debug, Clone)]
pub enum PropertyKind {
    CONCRETE,
    COMPUTABLE,
    ABSTRACT
}

#[derive(Builder, Debug, Clone)]
pub struct Link<'a> {

    #[builder(setter(into))]
    pub ident:      String,

    #[builder(default = "false")]
    pub overloaded: bool,

    #[builder(default = "false")]
    pub abs:        bool,

    #[builder(default = "false")]
    pub readonly:   bool,

    #[builder(default = "false")]
    pub required:   bool,

    #[builder(default = "false")]
    pub multi:      bool,

    pub kind:          &'a LinkKind,

    #[builder(setter(into, strip_option))]
    pub default:       Option<&'a Expression>,

    #[builder(setter(into, strip_option))]
    pub constraints:   Option<Box<Vec<Constraint>>>,

    #[builder(setter(into, strip_option))]
    pub extends:       Option<&'a [RefCell<Weak<Link<'a>>>]>,

    #[builder(setter(into, strip_option))]
    pub module:        Option<RefCell<Weak<Module>>>
}

#[allow(dead_code)]
#[allow(non_snake_case)]
#[derive(Debug, Clone)]
pub enum LinkKind {
    CONCRETE,
    COMPUTABLE,
    ABSTRACT
}

#[derive(Builder, Debug, Clone)]
pub struct Constraint {
    
    #[builder(setter(into))]
    pub ident: String,

    #[builder(setter(strip_option), default)]
    pub extends: Option<RefCell<Weak<Constraint>>>,

    #[builder(default = "false")]
    pub delegated: bool,
 
    #[builder(default = "false")]
    pub on_abstract_types: bool,   

    #[builder(default = "false")]
    pub on_concrete_scalar_types: bool,

    #[builder(default = "false")]
    pub on_concrete_object_types: bool,

    #[builder(setter(strip_option), default)]
    pub args: Option<Box<Vec<ArgSpec>>>,

    #[builder(setter(strip_option), default)]
    pub subcommands: Option<Box<Vec<Subcommand>>>
}

#[derive(Builder, Debug, Clone)]
pub struct Expression {
    
    #[builder(setter(into))]
    pub ident: String

}

/// `UsingExpression`:
///    A boolean expression that returns true for valid data 
///    and false for invalid data. 
///    The expression may refer to the subject of 
///    the constraint as __subject__.
type UsingExpression = Expression;

#[derive(Builder, Debug, Clone)]
pub struct Subcommand {
    
    #[builder(setter(strip_option), default)]
    pub using: Option<Box<UsingExpression>>,

    #[builder(setter(strip_option, into), default)]
    pub err_message: Option<String>,

    #[builder(setter(strip_option), default)]
    pub annotation: Option<Box<Annotation>>

}

#[derive(Builder, Debug, Clone)]
pub struct Index {

    #[builder(setter(into))]
    pub ident: String,

    #[builder(setter(strip_option), default)]
    pub alias: Option<Annotation>

}

#[derive(Builder, Debug, Clone, PartialEq)]
pub struct Function<R, S> 
    where R: FuncRet,
          S: FuncScope + FuncRet
{
    #[builder(setter(into))]
    pub ident: String,

    #[builder(setter(strip_option))]
    pub args:  Option<Box<Vec<ArgSpec>>>,

    #[builder(setter(into, strip_option))]
    pub ret_type: Option<Box<R>>,

    #[builder(setter(strip_option))]
    pub scope: Option<Box<S>>
}


pub trait FuncRet {

}

pub trait FuncScope {

}
