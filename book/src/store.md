# Store

Every package that is installed in hua will be saved in the store. This allows different profiles
to use the same store and dramatically simplifies our caching system. Also this opens up the
possibility to use a local store accross multiple installations in the future.
The Store is placed under `/hua/store` and every package is stored under firstly its package name
like `/hua/store/firefox` and further is specified by its version number and hash:
`/hua/store/firefox/95.0.1-pwiq2139q92s1m3rhefw310`. This allows us to uniquely identify each package.

## Commands

#### Garbage Collection

This will search for each package in the `/hua/store` if it is still in use by any
generation of all profiles. If it is not in use anymore, it will be deleted.

```bash
hua store collect-garbage
```

Note for implementation:
Depending on the implementation this may be slow, maybe evaluate using a sqlite db for this.

## Roc

The removal of unsued packages can be automated in the system config.

```elm
app "config"
    packages { pf: "hua" }
    imports [ pf.StoreConfig ]
    provides [ store-config ] to pf

store-config : StoreConfig
store-config = { collect-garbage-after: "7d" }
```
