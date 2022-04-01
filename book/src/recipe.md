## Requirements

- version requirement e.g. `>1.0`
- name
- components: the files that are needed

#### Caveats

A package that matches those requirements can easily be not functioning anyway, because of user patches or because a future release is not backwards compatible.

A possible solution for this would be to allow the user to specify a concrete dependency(and its hash) with which the package is known to run (or calculate them when building and then distribute them as .lock file) and then
run the package in a chrooted/sandboxed environment.

#### Permissions

- only root users can build a recipe into the store
- we do that by pam
- we do not need fakeroot, as packages have to be built into the store when distributing them and the store is always root only