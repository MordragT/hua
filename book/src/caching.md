# Caching

As hua is not differentating between binary and source packages a caching sytem may be benefitial,
to speed up the process of installation for the end user.
As every package in the hua store can be uniquely identified by its appending hash, we can just
host a fileserver of our own store.
The caching servers can be prioritized from 1 (lowest) to 10
and the configuration is saved under `/hua/config/caches.toml`.
As caching is optional and will not change the outcome of an installation, the configuration can
also be changed by hand and it will not create a new generation.

## Commands

#### Add

This adds a caching server.

```bash
hua cache add <local | ssh | http | https> <prio>?
```

#### Remove

This searches the available caching servers and asks which to remove.

```bash
hua cache remove <name>

Which caching server do you want to remove ? (abort with ctrl+c)
> option1
option2
```

#### List

This lists all active caching servers with their corresponding priorization.

```bash
hua cache list
```

## Roc

The caching servers can also be specified in the central system config.

```elm
app "config"
    packages { pf: "hua" }
    imports [ pf.Cache ]
    provides [ caches ] to pf

# TODO: check if it is really possible to import remote files
caches : List Cache
caches =
    local = { root: "/srv/hua-cache/", priorization: 10 }
    [ local ] 
```

## Create youre own caching server

Just copy youre `/hua/store` to a web file server, or use the `/hua/store` directly as cache.