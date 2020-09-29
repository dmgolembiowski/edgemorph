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

#### but now — add the `edgemorph.toml` file to your project

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

#### Rust API

```rust
use edgemorph::{Type, Property, Link}

#[derive(Type)]
#[type(abstract="true")]
struct Named {
    #[property("str", required="true")]
    name: Property<String>
}

#[derive(Type)]
#[type(abstract="true")]
struct HasAddress {
    #[property("str")]
    address: Property<String>
}

#[derive(Type)]
#[type(extending=("Named", "HasAddress")]
#[index("name")]
struct User {
    #[link("User"), multi="true")]
    friends: Link<User>
}
```


#### Python API

```python
from edgemorph import ( edgetype, property, link, multi )

@edgetype(abstract=True)
class Named:
    name: property[str]

@edgetype(abstract=True)
class HasAddress:
    address: property[str]

@edgetype(extending=(Named, HasAddress))
class User:
    friends: multi[ link[__qualname__] ]
    index:   {
        "name": lambda title : "User name index"
    }
```

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


