# stringify-err

`Result<(), impl std::error::Error>` -> `Result<(), String>`

Mainly useful for FFI.

Not yet on crates.io. It is currently very quick and dirty (a lot of code is from wrap-match). It has no UI tests. Before it is published to crates.io, the code will be improved, more tests will be
added and configuration options will most likely be added.
