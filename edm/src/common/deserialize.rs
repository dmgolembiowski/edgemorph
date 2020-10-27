#### `TreeNode`
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

