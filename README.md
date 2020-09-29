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

Unlike traditional object-relational mappers, Edgemorph requires users to write database level code in EdgeQL (EdgeDB Query Language) so that it can be compiled into a bytecode library target. Additionally, the compiler returns pre-baked files in [any of the supported programming languages]() so that project-level code can communicate with EdgeDB databases using common ORM coding patterns. For instance, `edm compile -f user.edgeql` would compile the user module file and return both a library with a `.so` extension, in addition to files in [the supported language targets]() i.e. Rust, Python, or both, with typical ORM methods each of the types.

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


