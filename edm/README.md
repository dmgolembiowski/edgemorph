# edm: edgemorph development manager
_the command line tool for the edgemorph framework_

***

## Formal specification

***

* **`edm init`** [ _directory_name_ | _._ ] 
> _Initializes an edgemorph project in the `directory_name` specified. If `directory_name` is neither empty nor the `.` character, and the directory does not already exist, then this call creates the directory. Otherwise, the current working directory is assigned by default. This value becomes the `project_root` variable. Next, the command creates the `edb_modules` folder under the `project_root`. Then, `$project_root/edb_modules/mod_${project_root}.esdl` is created and pre-populated with:_

```sql
module ${project_root} {

}
```

> _Finally, `$project_root/edgemorph.toml` is created. It is pre-populated with:_

```toml
[edgemorph]
project_root    = "{project_root}"
mod_directories = ["/edb_modules"]

[edgemorph.codegen]
schema_name = "{schema}"

[edgemorph.codegen.rust]
enabled = "true"

[edgemorph.codegen.rust.modules]
    [edgemorph.codegen.rust.modules.{project_root}]
    source = "mod_{project_root}.esdl"
    output = "{project_root}/src/lib/edm_{project_root}.rs"

[edgemorph.codegen.python]
enabled = "true"

[edgemorph.codegen.python.modules]
    [edgemorph.codegen.python.modules.{project_root}]
    source = "mod_{project_root}.esdl"
    output = "/{project_root}/edm_{project_root}.py"

[edgedb]
[edgedb.databases]
[edgedb.databases.primary]
name = ""
dsn = ""

[edgedb.databases.primary.modules]
MODULE_NAME = "/edb_modules/mod_{project_root}.esdl"
```

***

* **`edm add`** [ _new_module_ ]
> _If `new_module` is present on the filesystem and does not conflict with any module names already allocated within `edgemorph.toml`, then its information is written to `edgemorph.toml` and is registered with each of the possible database names._

*** 

* **`edm make`** [ _edb_module_ | * ]
> _Retrieves each of the module files from `edgemorph.toml` and if `edb_module` is non-empty and in the set of registered modules, this call forwards the `edb_module` to the lower-level call `edm compile`. Alternatively, if `edb_module` is empty then this same procedure is done for each of the registered module files. If compilation is successful, this process writes a file to disk with the edgemorph-augmented SDL migration. Then, the command informs the user to run `edm make install` can be run to compile the EdgeQL source to local package targets. Otherwise, this process prints the syntactical errors emitted by `edgeql-parser` and exits._

***

* **`edm compile`** [ _edb_module_path_ ]
> _Panics when `edb_module_path` is not a valid target. Otherwise, runs the source module through a single-pass compilation process leveraging the `edgeql-parser`. When the output matches `Result<T>`, `edm compile` digests the AST tokens into static Edgemorph datastructures with trait implementations for calling module-level prepared at the database level and separate implementations for binding public-facing methods to EdgeQL functions that can queried directly on the database.

*** 

* **`edm test`** [ _edgedb_ident_ ] [ _database_name_ ]
> _Panics when `edgedb_ident` and `database_name` are not jointly available in `edgemorph.toml`. Otherwise, checks whether connectivity can be established between edgemorph and the `edgedb_ident` for `database_name`._
***

* **`edm make install`** [ (_edb_module_)+ | * ]
> _Checks `edgemorph.toml` for the `edgedb_ident` and `database_name` pairs associated with `edb_module`. Panics if these correspondences are not registered. Otherwise, runs `edm test ${edgedb_ident} ${database_name}` for each of the relevant pairs. For each one that does not panic, this process starts a transaction with the databases and commits N migrations for each of the K-many SDL migrations stored in the `$project_root`._

***

