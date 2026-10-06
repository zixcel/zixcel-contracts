# zixcel-bounded-io (local integration)

Independent caller-budgeted physical input library in the zixcel-contracts repository.
The sibling zixcel-contracts wire crate does not depend on this crate and retains external-actions=false.
This crate can open/read files and directories and owns no writes, locking, authorization, defaults or semantics.
Publication remains disabled. Unix and Windows regular-file intake. Windows uses standard reparse-point open flags and safe disk-handle inspection; no-follow is final-component only.
Final-component no-follow does not establish ancestor confinement. Original LICENSE/NOTICE are retained.

Regular-file protection applies to the opened final handle; it does not confine ancestors.
Directory enumeration uses symlink_metadata followed by std::fs::read_dir. The existing
check/use race remains: a directory can be replaced between those operations. This API
has no atomic no-follow directory handle and makes no race-free enumeration guarantee.
