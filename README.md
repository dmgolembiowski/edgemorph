### ⚠️ WIP ⚠️ 
##### _Notice: This project is under development and is not appropriate for production projects._ 

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
project_root    = "edgedb_app"
mod_directories = ["/edb_modules"]

[edgemorph.codegen]
schema_name = "Edgemorph"

[edgemorph.codegen.rust]
enabled = "true"

[edgemorph.codegen.rust.modules]
    [edgemorph.codegen.rust.modules.edgedb_app]
    source = "/edb_modules/edgedb_app.esdl"
    output = "/src/lib/edm_edgedb_app.rs"

[edgemorph.codegen.python]
enabled = "true"

[edgemorph.codegen.python.modules]
    [edgemorph.codegen.python.modules.edgedb_app]
    source = "/edb_modules/edgedb_app.esdl"
    output = "/edgedb_app/edm_edgedb_app.py"

[edgedb]
[edgedb.databases]
[edgedb.databases.primary]
name = ""
dsn = ""

[edgedb.databases.primary.modules]
edgedb_app = "edb_modules/edgedb_app.esdl"
```

Unlike traditional object-relational mappers, Edgemorph requires users to write database-level code. Using EdgeDB's rich query language, Edgemorph combines the strictly typed qualities of EdgeDB with a library-factory written in Rust. This unconventional strategy allows users to compile entirely custom bytecode libraries on a per-project basis, but continue to program in the stylings of a typical ORM.

For instance, if we have an EdgeDB module within the `user.esdl` file, then executing `edm compile -f user.esdl` would compile the user module to return both a dynamic library, `edm_user.so`, as well as a native code file in Rust or Python for reaching it. Currently, I am only planning to support Rust and Python  API outputs, however I would like JavaScript, either via `wasm32` architecture or pure JS, to join the adventure as well.

Here are examples of what the compiler generates in Rust and Python given the EdgeQL above.

#### Rust API Output

```rust
// user.rs

use ::edm_user::{NamedType, UserType, HasAddressType};
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
# user.py

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

#### Development / Build Instructions for `edm`

_Note: Pre-requisite dependencies include having
Python 3.8+, Poetry, Git, and Rust nightly installed._

```bash
# Have Python 3.8+, Poetry, and Rust nightly installed
git clone https://github.com/dmgolembiowski/edgemorph.git
cd edgemorph
git submodule update --init --recursive
cd edm
poetry shell
poetry install
cd bootstrap/edgedb
python -m pip install -v -e .

# And viola! You can now run `edm` based commands.
```
## Roadmap (incomplete)
***
### EDM Features
- [X] `edm init`
- [ ] `edm make` 
- [X] simple offline SDL syntax checker
- [X] hidden AST dump to EDB modules file
- [X] "makeability" protected by project-level configuration file `edgemorph.toml`
- [ ] single and (concurrent) multi-file lex checking
- [ ] `edm add`
- [ ] `edm make install`
- [ ] `edm test`

In the future, I would like to see `edm` support multi-language target compilation so that changes to the native programming language code can result be retrofitted onto the original shema with either DDL modifications or 1-to-1 SDL modifications.

