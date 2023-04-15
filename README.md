# `oid`

An id scheme. Why another one?

- `lexicographically sortable` - `uuidv4` is not sorted and will explode database indexes. Like `ulid`, `oid` is lexicographically sortable.
- named - oid can be prefixed with a name, like `team_0da0fa0e02cssbhkanf04c_srb0`
- short-codable - oid can be written in shortform like `team_srb0`
- double-click-copyable - try double clicking this uuid: `a827f03c-f5b0-40ef-8d53-3fb3cdf4e055`. Then try this `oid`: `team_0da0fa0e02cssbhkanf04c_srb0`

# Usage

```rust
use oid::{new_oid, label};

label!(Team, "team");
label!(User, "usr");
label!(Transaction, "tx");

fn main() {
    let id = Team::oid();
    // e.g. id: team_0da0fa0e02cssbhkanf04c_srb0
    println!("id: {}", id);
    // e.g. id: team_srb0
    // e.g. uuid: a827f03c-f5b0-40ef-8d53-3fb3cdf4e055
    println!("short: {}", id.short());
    println!("uuid: {}", id.uuid());
    
    // We didn't use a Label, so it's simply missing.
    let id = new_oid();
    // e.g. id: 0da0fa0e02cssbhkanf04c_srb0
    println!("id: {}", id);
}
```

# Installation

```toml
[dependencies]
oid2 = "*"
```

Despite `oid2` as the package name, you still import it as `use oid::{}`.

# Roadmap

- [ ] Create a postgres extension to store oids as u128, but have it display in human readable form.