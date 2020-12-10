use edgedb_derive::Queryable;
use std::collections::{BTreeSet};
use std::sync::{Arc};
use std::mpsc::Mutex;
use std::rc::Weak;
use std::cell::RefCell;

pub struct Arena<T>(Arc<Mutex<Weak<RefCell<BTreeSet<T>>>>);

#[derive(Queryable, Debug)]
pub struct IRObject<Ptr>
    where Ptr: FnMut(&IRId) -> Arena<IRObject> 
{
    id: IRId,
    cardinality: IRCardinality,
    name: IRName,
    is_abstract: IRIsAbstract,
    enum_values: IREnumValues,
    material_id: IRMaterialId,
    bases: IRBases,
    ancestors: IRAncestors,
    union_of: IRUnionOf,
    intersection_of: IRIntersectionOf,
    pointers: Ptr,               // Callback to get interior mutable set of IRObject(s) from `&self.id`
    array_element_id: Option<u32>,
    tuple_elements: todo!(),     // TODO: IRTupleElements
    required: IRRequired,
    expr: IRExpr,
    target_id: Option<String>
}

#[derive(PartialEq, Eq, Debug)]
pub struct IRId(Option<String>);

#[derive(PartialEq, Eq, Debug)]
pub struct IRCardinality(Option<IRCardinalityKind>);

#[derive(Eq, PartialEq)]
pub enum IRCardinalityKind {
    Any,
    Many,
    One,
}

#[derive(PartialEq, Eq, Debug)]
pub struct IRName(Option<String>);

#[derive(PartialEq, Eq, Debug)]
pub struct IRIsAbstract(Option<bool>);

#[derive(PartialEq, Eq, Debug)]
pub struct IREnumValues(Option<Vec<String>>);

#[derive(PartialEq, Eq, Debug)]
pub struct IRMaterialId(Option<String>);

#[derive(PartialEq, Eq, Debug)]
pub struct IRBases(BTreeSet<IRId>);
/*
// https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=f7da1e5a05c35f6ac6e99f7e421b88ca
macro_rules! vecomp {
    [$expr:expr; for $pat:pat in $iter:expr $(; if $cond:expr )?] => {
        IntoIterator::into_iter($iter)
            $(
                .filter(|$pat| $cond)
            )?
            .map(|$pat| $expr)
            .collect::<Vec<_>>()
    }
}

let actual = vecomp![x + x; for x in 1..50; if *x % 2 == 0];
*/


// Functionally, these types are similar enough
// on the EdgeDB side to warrant a shared trait
// for builder functionality.
// 
//
pub struct IRAncestors(BTreeSet<IRId>);
pub struct IRUnionOf(BTreeSet<IRId>);
pub struct IRIntersectionOf(BTreeSet<IRId>);

pub struct IRKind(Option<ObjKindIdent>);

impl IRKind {
    pub fn property() -> IRKind { IRKind(Some(ObjKindIdent::Property))}
    pub fn object()   -> IRKind { IRKind(Some(ObjKindIdent::Object))}
    pub fn scalar()   -> IRKind { IRKind(Some(ObjKindIdent::Scalar))}
    pub fn link()     -> IRKind { IRKind(Some(ObjKindIdent::Link))}
    pub fn tuple()    -> IRKind { IRKind(Some(ObjKindIdent::Tuple))}
    pub fn none()     -> IRKind { IRKind(None)}
}

impl From<Option<&str>> for IRKind {
    fn from(string: Option<&str>) -> Self {
        match string {
            Some("property") => IRKind::property(),
            Some("object")   => IRKind::object(),
            Some("scalar")   => IRKind::scalar(),
            Some("link")     => IRKind::link(),
            Some("tuple")    => IRKind::tuple(),
            _                => IRKind::none()
        }   
    }   
}

impl From<&str> for IRKind {
    fn from(string: &str) -> Self {
        match string {
            "property" => IRKind::property(),
            "object"   => IRKind::object(),
            "scalar"   => IRKind::scalar(),
            "link"     => IRKind::link(),
            "tuple"    => IRKind::tuple(),
            _          => IRKind::none() // Should throw/log error
        }   
    }   
}

#[derive(Eq, PartialEq)]
pub enum ObjKindIdent {
    Property,
    Object,
    Scalar,
    Link,
    Tuple
}


