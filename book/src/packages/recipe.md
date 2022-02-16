# Recipe

Goals:
- automatic hashing and downloading of the files
- no hardcoded install pathes

Idea:
The roc file that is executed creates a .lock(toml) file with hashes and stuff.
This lock file is required to install a package, the recipe is not.
The lock file can also be created manually, but it is more convenient with roc
(as the platform is written in rust anyway, also rust possible?).

AnotherIdea:
A roc file is all you need to add a package to a store or install it into the generation.
It will create a `package.db` file inside the store (or local build folder) which is used by the generation
to create its `packages.db` file to track installation files and etc.

3rd idea:
A roc file is needed, which will create a recipe.lock file from the inputs it downloaded (the roc platform will automatically add
the lock file on downlading data as the download functions are all defined by the platform aswell).
#### Lock File

```toml

```

---

A recipe defines where to download the package content and how to install it.
This process is defined in different stages.

## Example package

```elm
# recipe.roc

app "config"
    packages { pf: "hua-package" }
    # maybe import something like a prelude ?
    imports [ pf.PackageType, pf.Archive, pf.Folder, pf.Info, pf.File, pf.Path, pf.Architecture, pf.Recipe, pf.bin, pf.share, pf.cfg, pf.lib, pf.tmp ]
    # provides [ pf.Recipe ] to pf
    provides [ build_depends, depends, build ]

build_depends = []

depends : List Recipe
depends =
    # check if roc can import remote downloaded files, then we may use a Recipe type normally
    glibc <- await (Recipe.download "https://github.com/hua/hua-free/glibc/2.40.0/recipe.roc" "sha512-jfs0938hrndksy921nklasi23")
    [ glibc ]

build : Result Info
build = \arch, os
    when arch, os is
    X86_64, Linux => 
        helix_tar : File
        helix_tar <- await (Archive.download "https://github.com/helix-editor/helix/releases/download/v0.6.0/helix-v0.6.0-x86_64-linux.tar.xz" "sha512-k3k22lk238fnao92n2kmfem2i9jefn" tmp)
        
        helix_src : Path
        helix_src <- await (Archive.extract helix_tar tmp)
    
        _ <- await (Folder.write (helix_src ++ "usr/bin/helix") bin)
        
        OK { name: "helix"
        , version: "0.6.0"
        , desc: "An modal editor"
        , archs: [ X86_64 ]
        , licenses: [ "MPL2" ]
        , provides: "helix"
        , type: Binary
        }
    _ => NotSupported
```

---

# New

You can run the roc file like a script to create a entry in the store. In the working directory from which
you ran the script, a link to the result in the store is created. You can then also run `hua add` to add the
build to a generation, as the command will also look in the store for packages.

```elm
app "config"
    packages { pf: "hua-build" }
    imports [ pf.untar, pf.Info, pf.File, pf.Path, pf.Architecture, pf.Package, pf.Source ]
    provides [ package ] to pf
    
    source : Source
    source = Link "https://github.com/helix-editor/helix/releases/download/v0.6.0/helix-v0.6.0-x86_64-linux.tar.xz"
        
    glibc : PackageSource
    glibc = Git "https://github.com/hua/hua-free/glibc/2.40.0/recipe.roc"

    info : Info
    info =
        { name: "helix"
        , version: "0.6.0"
        , desc: "An modal editor"
        , archs: [ X86_64 ]
        , licenses: [ "MPL2" ]
        , provides: "helix"
        }

    package : Package
    package =
        files = { bin: [ "usr/bin" ], share: [], cfg: [], lib: [] }
        { info, files, source, dependencies: [ glibc ] }
```

Creates a recipe.lock when building, if it is not already there

```toml
name = "helix"
version = "0.6.0"
desc = "An modal editor"
archs = [ "X86_64" ]
licenses = [ "MPL2" ]
provides = "helix"
files = { bin = [ "usr/bin" ], share = [], cfg = [], lib = [] }
source = { root = "https://...", sha512 = "ASH33wfaQeSFgdrfdasfinq2q" }
dependencies = [ { name = "glibc, root = "https://..."} ]
```

```elm
app "config"
    packages { pf: "hua" }
    imports []
    provides [ config ] to pf

    config = \old_config ->
        { old_config & packages: old_config ++ package }


```