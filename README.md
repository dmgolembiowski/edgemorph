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
schema_dir        = ["<path to schemas>"]

[edgedb]
databases         = [ 
    [
        ["dsn", "<dsn_1>"], 
        ["database", "<database_name_1>"], 
        ["module", "<module_name_1>"]
    ],
    [
        ["dsn", "<dsn_2>"], 
        ["database", "<database_name_2>"], 
        ["module", "<module_name_2>"]
    ]
]
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
from edgemorph import ( edgetype, Property, MultiLink )

@edgetype(abstract=True)
class Named:
    name: Property[str]

@edgetype(abstract=True)
class HasAddress:
    address: Property[str]

@edgetype(extending=(Named, HasAddress))
class User:
    friends: MultiLink[__qualname__]
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


