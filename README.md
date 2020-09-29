![EdgemorphBanner](/banner.png)

#### Continue Writing EdgeQL as Normal

```sql
abstract type Named {
    required property name -> str;
}

abstract type HasAddress {
    property address -> str;
}

type User extending Named, HasAddress {
    # define some user-specific properties and a link
    multi link friends -> User;

    # define an index for User based on name
    index on (__subject__.name);
}
```

but now — your projects can leverage [`edm`](https://github.com/dmgolembiowski/edgemorph/tree/master/edm) to manage EdgeDB module schemas captured in a `edgemorph.toml`, where database modules are configured at the project-level.

```toml
[edgemorph]
enable_rs_binding = "true"
enable_py_binding = "true"
project_root      = ["<this file's parent directory>"]
mod_directories   = [
    "<relative path to dirA>",
    "<relative path to dirB>",
    "<relative path to dirC>",
    "..."
]
edgemorph_output  = {
    rust = {
        ["module_file_A", "</path/to/output_A>.rs"]
    },
    python = {
        ["module_file_B", "</path/to/output_B>.py"],
        ["module_file_C", "</path/to/output_C>.py"]
    }
}

[edgedb]
databases         = { 
    database_name_1 = [{
        dsn     = "<dsn_1>", 
        modules = ["<module_name_a>"]
    }],
    database_name_2 = [{
        dsn     = "<dsn_2>", 
        modules = ["<module_name_b>", "<module_name_c>"]
    }]
}
```

Unlike traditional object-relational mappers, Edgemorph requires users to write database-level code. Using EdgeDB's rich query language, Edgemorph combines the strictly typed qualities of EdgeDB types with a powerful library-factory written in Rust. This unconventional strategy allows users to compile entirely custom bytecode libraries on a per-project basis, but continue to program in the stylings of a typical ORM.

For instance, if we have an EdgeDB module within the `user.edgeql` file, then executing `edm compile -f user.edgeql` would compile the user module to return both a library with a `edm_user.so` extension as well as any application code files for calling these ORM methods. Currently, I am only planning to support Rust and Python  API outputs, however I would like JavaScript to join the adventure as well.

Here are examples of what the compiler generates in Rust and Python given the EdgeQL above.

#### Rust API Output

```rust
use crate::edm_user::{NamedType, UserType, HasAddressType};
use edgemorph::*;

#[derive(NamedType)]
#[type(abstract="true")]
struct Named {
    #[property("str", required="true")]
    name: Property<String>
}

#[derive(HasAddressType)]
#[type(abstract="true")]
struct HasAddress {
    #[property("str")]
    address: Property<String>
}

#[derive(UserType)]
#[type(extending=("Named", "HasAddress")]
#[index("name")]
struct User {
    #[link("User"), multi="true")]
    friends: Link<User>
}
```


#### Python API Output

```python
from edgemorph import ( edgetype, property, link, multi )
from .edm_user import ( NamedType, HasAddressType, UserType )

@edgetype(abstract=True, edb=NamedType)
class Named:
    name: property[str]

@edgetype(abstract=True, edb=HasAddressType)
class HasAddress:
    address: property[str]

@edgetype(extending=(Named, HasAddress), edb=UserType)
class User:
    friends: multi[ link[__qualname__] ]
    index:   {
        "name": lambda title : "User name index"
    }
```

In the future, I would like to see `edm` support multi-language target compilation so that changes to the native programming language code can result be retrofitted onto the original shema with either DDL modifications or 1-to-1 SDL modifications.

### Installation

(Installation steps for Rust-API)

```
_
_
_
```

(Installation steps for Python-API)

```
_
_
_
```


