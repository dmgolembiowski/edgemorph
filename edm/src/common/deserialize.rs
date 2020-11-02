use pest::iterators::{Pairs, Pair};
use pest;
use std::fs::File;
use std::io::{BufReader};
use std::io::prelude::*;

/// Derived from the official EdgeDB repository at `edb/edgeql/ast.py`
///
/// AST Nodes
/// ---------
///
/// `TreeNode`
/// - `id`: `<i32>`
/// - `name`: _N<sub>T</sub>_ such that _N<sub>T</sub>_ &#8712; _T **&acute;**_ and  _T **&acute;**_ satisfies the size requirements for each of the following identifiers. :
/// > ```python  
/// > { 'BinOp', 'CreateConcreteLink', 'CreateConcreteProperty',
/// > 'CreateFunction', 'CreateObjectType', 'ForQuery',
/// > 'FuncParam', 'FunctionCall', 'FunctionCode',
/// > 'InsertQuery', 'IntegerConstant',
/// > 'ModuleAliasDecl', 'ModuleDeclaration',
/// > 'ObjectRef', 'Path', 'Ptr',
/// > 'Schema', 'SelectQuery', 'Set',
/// > 'SetField', 'ShapeElement', 'ShapeOperation',
/// > 'StringConstant', 'TypeCast', 'TypeName' }  
/// > ```
/// - `children`: `CheckedList<TreeNodeChild, Markup>`
/// ***
/// #### `TreeNodeChild`
/// - `id`: `Optional<i32>`
/// - `label`: `String` _s_, such that _s_  &#8712; _L_ &equals; `{ "name", "target", "maintype" }`
/// - `node`: `enum <String ; TreeNode; List >`, with the following corollaries:
///   -  `node::String` &#8594; `<String str = '%s'>`;
///   - `node::TreeNode` &#8594; `&'a Sized<RefCell<Weak<TreeNode<'a>>>>`. **_'a_** is the lifetime specifier for the `TreeNode` it ellides. `Sized<T>` is a type with known size. `RefCell<T>` is a mutable memory location with dynamically checked borrow rules<sup>1</sup>. `Weak<T>` is a pointer that holds a non-owning reference to the managed allocation<sup>2</sup>. [`TreeNode`](https://github.com/edgedb/edgedb/blob/ff869ab54d63968081d54e99b4dfb8f2b62f9ce8/edb/common/markup/elements/lang.py#L81) is an EdgeDB language markup base object subtype.
/// 
/// ***
/// #### `Schema`
/// - `declarations`: `Vec<ModuleDeclaration>`
/// ***
/// #### `ModuleDeclaration`
/// - `name`: `ObjectRef<&str>`
/// - `declarations`: `Vec<Declaration>`
/// ***
/// #### `Declaration`
/// - `CreateObjectType`:
///   ```rust
///   name: String,
///   commands: Vec<CreateConcreteProperty> | Vec<CreateConcreteLink>
///   ```
/// - `CreateFunction`:
///   ```rust
///   name: String
///   params: Vec<FuncParam>
///   returning: Optional<TreeNodeChild>
///   returning_typemod: Optional<TreeNodeChild>
///   ```
/// 


/// Seek to given rule in a sequence of pairs.
///
/// Usage:
/// 
///       let a_prefix = String::from(seek_to(pairs, &Rule::TEXT).unwrap().as_str());
///
///       let int_id = String::from(seek_to(pairs, &Rule::LONG).unwrap().as_str()).parse::<u64>().unwrap();
///
/// Credits to: PetrGlad (https://github.com/pest-parser/pest/issues/405#issue-481147665)
pub fn seek_to<'a, R: pest::RuleType>(
            pairs: &mut Pairs<'a, R>, to: &R ) -> Option<Pair<'a, R>> 
{
    for p in pairs {
        if p.as_rule() == *to {
            return Some(p);
        }
    }
    None
}

/// Find first pair at given rule path.
///
/// Usage:
/// 
///       let name = String::from(seek_in(pairs, &[Rule::Name, Rule::IdOrReservedWord, Rule::ID]).unwrap().as_str());
///
/// Credits to: PetrGlad (https://github.com/pest-parser/pest/issues/405#issue-481147665)
pub fn seek_in<'a, R: pest::RuleType>(
                pairs: &mut Pairs<'a, R>, 
                to_path: &[R]) -> Option<Pair<'a, R>>
{
    match to_path {
        [] => None,
        [r] => seek_to(pairs, r),
        rs => match seek_to(pairs, &rs[0]) {
            Some(r) => seek_in(&mut r.into_inner(), &to_path[1..]),
            None => None
        }
    }
}

fn load_ast_file(path: &str) -> std::io::Result<String> {
    let file = File::open(path.to_owned())?;
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents)?;
    Ok(contents)
}



