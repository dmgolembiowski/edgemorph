# Edgemorph
*An EdgeDB Manipulator of Relational, Polymorphic Hierarchies*
<br />

### Synopsis
(Motivations for this library)

### Usage

( Describe the importance of manipulating the object hierarchy with a fluid schema that does not compromise runtime performance )

#### EdgeQL Schema Definition

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


