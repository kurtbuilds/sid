# `sid`

An id scheme. Why another one?

- **Lexicographically sorted** - The first 40 bits of `sid` are a second precision timestamp, which can be lexically 
  sorted as strings (similar to `ulid`). This keeps database indexes small, unlike `uuid` which can explode index size.
- **Named** - sids can be labeled, e.g. `usr_0da0fa0e02cssbhkanf04c_srb0` or unlabeled.
- **Readable and memorable** - sid has a 4 character suffix, which is easy to remember, speak, and type, for quick
  visual and verbal comparison.
- **Double-click-copyable** - Try double clicking this uuid: `a827f03c-f5b0-40ef-8d53-3fb3cdf4e055`. Then try this
  sid: `team_0da0fa0e02cssbhkanf04c_srb0`
- **Compatible** - The data is a u128, making it interoperable with both `uuid` and `ulid` libraries.

When generating a random `sid`, data is generated using the same schema as (non-sequential) ulid, where the first 40 
bits are a timestamp, and the remaining 88 bits are random. 40 bits of a second precision timestamp means id generation
will wrap in 34,000 years.

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

In databases where you want to use the extension, you must create it:

```sql
CREATE EXTENSION sid;
```

# Installation

```toml
[dependencies]
sid2 = "*"
```

Despite `sid2` as the package name, you still import it as `use sid::{}`.

# Roadmap

- [x] Create a postgres extension to store sids as u128, but have it display in human readable form.
