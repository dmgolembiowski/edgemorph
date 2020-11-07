use derive_builder::Builder;
use std::rc::Weak;
use std::cell::RefCell;
use std::boxed::Box;
use map_vec::Set; // Contemplating an alternative.
use std::mem::ManuallyDrop;

#[derive(Debug, Clone, Default)]
pub struct Module {}

#[derive(Builder, Debug, Clone)]
#[builder(build_fn(validate = "Self::validate"))]
pub struct Type<'a, 'b, 'c, 'd> {

    #[builder(setter(into))]
    pub ident:       String,

    #[builder(setter(into), default = "false")]
    pub abs:         bool,
    
    #[builder(setter(into), default = "false")]
    pub scalar:      bool,
    
    #[builder(setter(into, strip_option), default)]
    pub extends:     Option<&'a [RefCell<Weak<SuperType<'b, 'c, 'c, 'd>>>]>,

    #[builder(setter(into, strip_option), default)]
    pub properties:  Option<Box<Vec<Property<'a>>>>,

    #[builder(setter(into, strip_option), default)]
    pub annotations: Option<Box<Vec<Annotation>>>,
    
    #[builder(setter(into, strip_option), default)]
    pub links:       Option<&'d [RefCell<Weak<Link<'d>>>]>,
    
    #[builder(setter(into, strip_option), default)]
    pub constraints: Option<Box<Vec<Constraint>>>,
    
    #[builder(setter(into, strip_option), default)]
    pub indices:     Option<Box<Index>>

}

impl<'a, 'b, 'c, 'd> TypeBuilder<'a, 'b, 'c, 'd> {
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

pub type SuperType<'a, 'b, 'c, 'd> = Type<'a, 'b, 'c, 'd>;
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

    // Kind has the longest lifetime. It probably should be static.
    // `'a` combines lifetime of `PropertyKind` and lifetimes of any
    // abstract `Property` this might extend. It's a possible memory leak
    // since any subtypes of an abstract Property might lead to references not
    // long enough (yikes).
    // I believe this could happen when some `Property`, p, 
    // satisfies p.abs == false and `if let Some(P) = p.extends.unwrap()` where `P.kind`
    // Some(P) where `P.kind` either == or != `self.kind`. If `self.kind != P.kind`, then 
    // we're screwed.
    pub kind:          &'a PropertyKind,

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
// impl ManuallyDrop for Expression 
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
pub struct Function<'poly, R, S> 
    where R: FuncRet<'poly>,
          S: FuncScope + FuncRet<'poly>
{
    #[builder(setter(into))]
    pub ident: String,

    #[builder(setter(strip_option))]
    pub args:  Option<Box<Vec<ArgSpec>>>,

    #[builder(setter(into, strip_option))]
    pub ret_type: Option<Box<&'poly R>>,

    #[builder(setter(strip_option))]
    pub scope: Option<Box<&'poly S>>
}

/// `edgemorph::FuncRet<'c>`
/// 
/// `FuncRet<'poly>` elides a second-order lifetime bound, such that
/// `'poly` must outlive both `'a + 'b + 'c`: the lifetime of both a `Type`'s or 
/// `self.extends` supertype, in addition to `'d`
/// `Link`'s references to 
pub trait FuncRet<'poly> {
    fn from<'a: 'poly, 'b: 'poly, 'c: 'poly, 'd: 'poly>(ty: &'c Type<'a, 'b, 'c, 'd>) -> Set<Type<'a, 'b, 'c, 'd>>;
}


#[repr(C)]
pub union Statement {
    pub(crate) uxpr: ManuallyDrop<UsingExpression>,
    pub(crate) axpr: ManuallyDrop<AliasExpr>,
    pub(crate) expr: ManuallyDrop<Expression>,
    pub(crate) subc: ManuallyDrop<Subcommand>,
    pub(crate) cons: ManuallyDrop<Constraint>
}

impl Drop for Statement {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.cons);
            ManuallyDrop::drop(&mut self.expr);
            ManuallyDrop::drop(&mut self.subc);
            ManuallyDrop::drop(&mut self.axpr);
            ManuallyDrop::drop(&mut self.uxpr);
        }
    }
}

pub trait FuncScope {
    fn from(stmts: &[Statement]) -> Vec<Box<Self>>;
}

