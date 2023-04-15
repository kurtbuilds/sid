# `oid`

An id scheme. Why another one?

- `lexicographically sortable` - `uuidv4` is not sorted and will explode database indexes. Like `ulid`, `oid` is lexicographically sortable.
- named - oid can be prefixed with a name, like `team_0da0fa0e02cssbhkanf04c_srb0`
- short-codable - oid can be written in shortform like `team_srb0`
- double-click-copyable - try double clicking this uuid: `a827f03c-f5b0-40ef-8d53-3fb3cdf4e055`. Then try this `oid`: `team_0da0fa0e02cssbhkanf04c_srb0`

# Roadmap

- [ ] Create a postgres extension to store oids as u128, but have it display in human readable form.