# Package Commands

#### Add

This will search the store if a package is already installed but not in use
and also all cookbooks for the package name and will return a choice 
of packages with their versions aswell as the location they were found in.
The package is installed for the current user and a new generation is created

```bash
hua add firefox

Which package do you want to install ? (abort with ctrl+c)
firefox-94.1.0 found in store
> firefox-95.0.1 found in github.com/MordragT/hua-packages
firefox-93.2.2 found in github.com/MordragT/hua-packages
```

You can also install multiple packages in one command, where you will be ask sequentially
for every package which version you want to install.

```bash
hua add firefox steam lutris
```

You can specify which profile to choose as the following

```
hua add musl --profile system
```

Also you can specify if the package should only be added to the new generation after a reboot

```
hua add gnome-desktop --boot
```

#### Removal

This will search all installed packages in the current profile and generation under
`/hua/profiles/current/generation/config/packages.lock` and asks which to remove.
This does not delete the package in the store but only its links in the generation.
On completion a new generation is created.

```bash
hua remove firefox

Which package do you want to remove ? (abort with ctrl+c)
> firefox-95.0.1-pwiq2139q92s1m3rhefw310
```

You can also remove multiple packages in one command, where you will be as sequentially
which package wou want to remove

```bash
hua remove firefox steam lutris
```

You can specify which profile to choose as the following

```
hua remove musl --profile system
```


#### Search

This will search the store and all cookbooks for the package name and return a list of packages.

```bash
hua search <name>
```

#### List

This will list all packages installed in the current profile and generation

```bash
hua list
```

You can specify the profile and/or generation to use the following way:

```bash
hua list --profile system --generation 13
```

#### Reset

This resets a packages configuration. Note that this does not delete configuration that lives under
the home directory.

```bash
hua reset lutris
```

The command can also be used on multiple packages.

#### Run

Programs installed by hua can not be easily run from the binary, because the filesystem structure is different.
Therefor they have to be run by hua self which internally uses **bubblewrap** which adds some security features aswell.
Locally downloaded files can also be run with this command, in that case the user is asked which dependency bundle he
wants to bring into the environment.

```bash
hua run firefox
```

```bash
hua run ./webdesigner

To run this program you have to declare which dependencies you want to bring into scope: (abort with ctrl+c)
console bundle
> graphical bundle
none
```