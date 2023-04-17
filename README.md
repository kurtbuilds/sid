# `sid`

An id scheme. Why another one?

- **Lexicographically Sorted** - `uuidv4` is fully random and can explode the size of database indexes.
  Like `ulid`, `sid` is lexicographically sortable.
- **Named** - sids can be unlabeled, or prefixed with a name, like `team_0da0fa0e02cssbhkanf04c_srb0`
- **Readable and memorable** - sid has a 4 character suffix, which is easy to remember, speak, and type, for quick
  visual and verbal comparison.
- **double-click-copyable** - Try double clicking this uuid: `a827f03c-f5b0-40ef-8d53-3fb3cdf4e055`. Then try this
  sid: `team_0da0fa0e02cssbhkanf04c_srb0`
- **compatible** - The data is a u128, meaning it is interoperable with both `uuid` and `ulid` libraries.

When generating a random `sid`, data is generated using the same schema as (non-sequential) ulid, where the first 48 
bits are a timestamp, and the remaining 80 bits are random.

# Usage

```rust
use sid::{sid, label, Label, Sid, NoLabel};

label!(Team, "team");
label!(User, "usr");
label!(Transaction, "tx");

struct MyUser {
    id: Sid<Self>,
}

impl Label for MyUser {
    fn label() -> &'static str {
        "usr"
    }
}

fn main() {
    let id = Team::sid();
    // e.g. id: team_0da0fa0e02cssbhkanf04c_srb0
    println!("id: {}", id);
    // e.g. id: team_srb0
    // e.g. uuid: a827f03c-f5b0-40ef-8d53-3fb3cdf4e055
    println!("short: {}", id.short());
    println!("uuid: {}", id.uuid());

    // We didn't use a Label, so it's simply missing.
    let id = NoLabel::sid();
    // e.g. id: 0da0fa0e02cssbhkanf04c_srb0
    println!("id: {}", id);
  
    let user = MyUser { id: sid() };
    // e.g. user.id: usr_0da0fa0e02cssbhkanf04c_srb0
    println!("user.id: {}", user.id);
}
```

### Postgres

Note if you use the postgres extension, the label is capped at 8 bytes (ascii chars) in length.

You can install the extension with:

```
cd pg && just install
```

# Installation

```toml
[dependencies]
sid2 = "*"
```

Despite `sid2` as the package name, you still import it as `use sid::{}`.

# Roadmap

- [ ] Create a postgres extension to store sids as u128, but have it display in human readable form.