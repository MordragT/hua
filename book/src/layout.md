# Filesystem Hierarchy Layout

Hua does not follow the FHS (Filesystem Hierarchy Standard) but aims to be to a certain extend
compatible with it.

- bin/* -> hua/profiles/current/generations/current/\*/\*/bin/* hua/profiles/system/generations/current/\*/\*/bin/*
- boot
- etc/* -> hua/profiles/current/generations/current/\*/\*/etc/* hua/profiles/system/generations/current/\*/\*/etc/*
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
    - config
        - caches.toml
    - cookbooks
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
                - usr
                    - bin/firefox
                    - lib/glibc-2.40 -> hua/store/glibc/2.40.0-o12pefß19cnq3ffjd92jdjd
            - 93.2.2-12hd1932hfao53bdhsaa8j3
        - glibc
            - 2.40.0-o12pefß19cnq3ffjd92jdjd
        - nginx
            - 1.23.1-so3uj3kkajjfm93291jiqwq
                - usr/bin/nginx
                - etc/nginx/nginx.conf
    - profiles
        - current -> tom
        - system
            - generations
                - current -> 10
                - 10 -> ...
        - tom
            - generations
                - current -> 23
                - 17
                - 23
                    - config
                        - cookbooks.lock
                    - packages
                        - firefox/95.0.1-pwiq2139q92s1m3rhefw310 -> hua/store/firefox/95.0.1-pwiq2139q92s1m3rhefw310
- lib/* -> hua/profiles/current/generations/current/\*/\*/lib/* hua/profiles/system/generations/current/\*/\*/lib/*
- root
- run
- sbin/* -> hua/profiles/current/generations/current/\*/\*/sbin/* hua/profiles/system/generations/current/\*/\*/sbin/*
- usr/* -> hua/profiles/current/generations/current/\*/\*/usr/* hua/profiles/system/generations/current/\*/\*/sbin/*
- var

This allows hua to be installed on many linux distributions and behave in an expectable manner