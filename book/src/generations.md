# Generations

The concept of generations allows hua to **rollback** each mutation-transaction.
Whenever a new package is installed or a new configuration is loaded, hua will create a new generation
for the profiles, for example:

```bash
hua install firefox-95.0.1 
```

This will create a new generation including the newly installed package.
Each generation keeps track of the packages installed aswell as the cookbooks used to install the packages.
These informations are stored under `generations/23/etc/hua/cookbooks.lock` and `generations/23/etc/hua/packages.lock`.
All the contents of a package that is installed is just linked in the generation, but depending on the 
configuration of the package specified in the recipe, some contents may also be copied.

## Commands

All following commands have an optional argument which specifies the profile
context under which the command shall be executed. By default the current profile
will be used. For example:

```bash
hua generations list --profile system
```

#### List

This will list all generations for the currently active user

```bash
hua generations list
```

#### Remove

This will remove all generations older than a threshold, but the current generations cannot be removed

```bash
hua generations remove <old(all except the current) | (number w(weeks) | d(days) | h(hours))>
```
---

Note:
Local Cookbooks are somewhat incompatible with this backup and restore aproach. I propose to delete this
feature as the system wide config offers a similar functionality

#### Backup

All information in hua which is required to create an exact copy of a generation is
saved by the filesystem itself. Therefor the following command creates a `caches.toml`,
`cookbooks.lock` and `packages.lock` under the current or specified directory. This files
can be used to restore the specified generation.

```bash
hua generations backup <generation-id>? <destination-path>?
```

#### Restore

With the `cookbooks.lock` and `packages.lock` files a generation can be restored. Optionally
a caches.toml file can be specified aswell. The restored generation is the new current generation.

```bash
hua generations restore <boot | switch> cookbooks.lock packages.lock caches.toml?
```