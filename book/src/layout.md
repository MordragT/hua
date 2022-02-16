# Filesystem Hierarchy Layout

Hua does not follow the FHS (Filesystem Hierarchy Standard) but aims to be to a certain extend
compatible with it.

- bin # all files(no folders) under ../generation/current/bin will be linked here
- boot
- etc # all files(no folders) under ../generation/current/cfg will be linked here
    - nginx
        - nginx.conf -> hua/user/current/generation/current/cfg/nginx/nginx.conf
- home
    - tom/Desktop/package-config
        - config.roc # the config can be placed anywhere
        - config.lock
        - cookbooks
            - local
                - hello-world
                    - recipe.roc
                    - recipe.lock
    - lisa
- hua
    - config -> hua/user/current/generation/current/etc/ & hua/user/system/generation/current/etc/
        - caches.toml
    - cookbook
        - github.com/MordragT/hua-packages-p12ijrnfll3ijf082neq234t5
            - firefox
                - 95.0.1
                    - recipe.roc
                    - recipe.lock
            - glibc
                - 2.40.0
                    - recipe.roc
                    - recipe.lock
    - store
        - firefox
            - 95.0.1-pwiq2139q92s1m3rhefw310
                - bin
                    - firefox
                - lib
                    - glibc-2.40 -> hua/store/glibc/2.40.0-o12pefß19cnq3ffjd92jdjd 
            - 93.2.2-asi2im4o1fß1idß3i1ndß1kd
        - glibc
            - 2.40.0-o12pefß19cnq3ffjd92jdjd
        - nginx
            - 1.23.1-so3uj3kkajjfm93291jiqwq
                - bin
                    - nginx
                - cfg/nginx
                    - nginx.conf # marked as configurable
    - user
        - current -> tom
        - system
            - generation
                - current -> 10
                - 10 -> ...
        - tom
            - generation
                - boot # will be used as new current after next boot
                - current -> 23
                - 17
                - 23
                    - bin
                        - firefox -> hua/store/firefox/95.0.1-pwiq2139q92s1m3rhefw310/usr/bin/firefox
                        - nginx -> hua/store/nginx/1.23.1-so3uj3kkajjfm93291jiqwq/usr/bin/nginx
                    - cfg
                        - hua
                            - cookbooks.lock # maybe combine both files -> generation.db
                            - packages.lock # maybe sqlite db instead of toml ? 
                        - nginx
                            - nginx.conf # when a program is started all the "configurable parts" are searched here
                    - lib
                        - glibc-2.40 -> hua/store/firefox/95.0.1-pwiq2139q92s1m3rhefw310/usr/lib/glibc-2.40
                    - share
                        - applications
                            - firefox.desktop # dekstop files must be altered to execute with hua run
- lib # all files(no folders) under ../generation/current/lib will be linked here- root
- run
- sbin
- usr
    - bin # all files(no folders) under ../generation/current/bin will be linked here
    - lib # all files(no folders) under ../generation/current/lib will be linked here
    - share # all files(no folders) under ../generation/current/share will be linked here
- var

## How does hua work ?

- distinction between binary and library packages to encourage packagers to not ship binaries with their libaraies (avahi)

#### Adding a package to the store

1. Fetch recipe (installation script and lock file) from a cookbook.
2. A hash is calculated out of the recipe
3. All remote caching servers are searched for the package name, version and hash
4. If the package is found it is downloaded into the store -> ready.
5. If the package is not found the recipe downloads the packages files into a tmp directory
6. All dependencies are also added to the store using this steps
7. Dont do this anymore, as the dependencies are in the global scope later anyway: The dependencies are linked/copied into their target location
8. The recipe processes this files to make them compatible with the store
9. The temporary directory is moved under the package name, version and hash in the store

#### Adding a package to the generation

1. Copy the package.lock of the current generation
2. Add the package to it and the dependencies (under the packages table)
3. Add the cookbook if not already added and connect the packages to the appropiate cookbooks

#### Updating the generation

1. Read the package.lock file for all packages that have to be installed into the new generation
2. Create a temprorary file for the new generation with the subdirectories: bin, lib, cfg, share
3. Link all files of the packages in the respective folders (bin, lib, cfg, share)
4. If a link conflicts with another check the origin of both links
5. If the origin is equal skip and move on, else abort and report an error
4. Increment the generation counter and move the folder under `hua/user/<target>/generation/<counter>`
5. Depending on the installation mode link the boot or current folder to the newly created generation

#### Running a program

As the current generation of the current user is always linked into the different folders (etc, usr, bin, lib),
manual installed scripts that use `#!/usr/bin/python` instead of `#!/bin/env python` should work out of the box.
Also as all libraries and configuration files are mapped into the FHS structure, we do not need to patch the elf
files, they are just normally found:

1. We run `/bin/firefox`
2. The corresponding application in the store is started: `/hua/store/firefox/95.0.1-pwiq2139q92s1m3rhefw310/usr/bin/firefox`
3. The application searches in the normal folder hierarchy for the libraries
4. As these libraries are linked aswell it just finds them too.

#### Removing a package from the generation

1. The package is searched in the `packages.lock` of the generation
2. If no other packages are dependent on this package the package is deleted out of packages.lock
3. As the dependency files are owned by the package aswell, they are deleted automagically
4. Create a new generation with the new packages.lock file (see steps in update generation)

#### Garbage collecting the store

1. All generations of all users are searched for their packages.lock
2. A new temprorary package.lock is created containing all packages of all packages.lock
3. All packages not contained in this package.lock will be removed

#### Implications

This approach has some implications. Other than nix we are not able to allow packages with conflicting file names, as they
have to be linked in the global scope. They can safely sit to each other in the store, but only 1 package can be active
at a time. We ensure this by having the **provides** setting. We encourage to add the version number to binaries and libraries.
It might be possible to add the version number to older versions of a package automatically or provide a link without the version
number in the name to the newest package. This will be evaluated in the future.
Also as packages have the possibility to allow write access to certain configuration files, this files are copied and not linked,
to ensure our store can still be used as caching server. At the same time, some applications might search for the configuration
relative to the binary, which will cause an error or an empty config as we are using the `cfg` folder. For the time being those
applications should not allow to change the configuration and multiple versions of the package can be created with different configurations.
In addition to that we only use one `lib` folder, which means we will only support 64-bit architectures.
If on the installation of a package it is shown, that another package already occupies a file link, an error must be shown.
It should be sugessted to report that error upstream.
Configuration changes do not create a new generation, therefor a configuration change that creates a non functioning system
cannot be resolved by rolling back to a previous generation (We should create tools to allow to reset the configuration of installed
packages in the generation: for one package and for all).
Dependency programs and .desktop files are also shown in global scope not just user installed packages.

To use the store as cache, recipes have to bring the downloaded files in the right format.

#### Use recipe

1. Recipe downloads files
2. Recipe verifies hashes of files
3. Recipe moves files around to bring them into the right format
4. Recipe tells package manager to take over
5. Package manager moves files into store (optional)

#### Create recipe

1. Write recipe with download links
2. Run recipe to get hashes of files
3. Write hashes into recipe
4. goto Use recipe