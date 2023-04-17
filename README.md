# `sid`

An id scheme. Why another one?

- `lexicographically sortable` - `uuidv4` is not sorted and will explode database indexes. Like `ulid`, `sid` is lexicographically sortable.
- named - sid can be prefixed with a name, like `team_0da0fa0e02cssbhkanf04c_srb0`
- short-codable - sid can be written in shortform like `team_srb0`
- double-click-copyable - try double clicking this uuid: `a827f03c-f5b0-40ef-8d53-3fb3cdf4e055`. Then try this `sid`: `team_0da0fa0e02cssbhkanf04c_srb0`

# Usage

```rust
use sid::{new_sid, label};

label!(Team, "team");
label!(User, "usr");
label!(Transaction, "tx");

fn main() {
    let id = Team::sid();
    // e.g. id: team_0da0fa0e02cssbhkanf04c_srb0
    println!("id: {}", id);
    // e.g. id: team_srb0
    // e.g. uuid: a827f03c-f5b0-40ef-8d53-3fb3cdf4e055
    println!("short: {}", id.short());
    println!("uuid: {}", id.uuid());
    
    // We didn't use a Label, so it's simply missing.
    let id = new_sid();
    // e.g. id: 0da0fa0e02cssbhkanf04c_srb0
    println!("id: {}", id);
}
```

# Installation

```toml
[dependencies]
sid2 = "*"
```

Despite `sid2` as the package name, you still import it as `use sid::{}`.

# Roadmap

- [ ] Create a postgres extension to store sids as u128, but have it display in human readable form.