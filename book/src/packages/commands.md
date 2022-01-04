# Package Commands

#### Add

This will search all cookbooks for the package name and will return a choice 
of packages with their versions aswell as the location they were found in.
The package is installed for the current user and a new generation is created

```bash
hua add firefox

Which package do you want to install ? (abort with ctrl+c)
> firefox-95.0.1 found in github.com/MordragT/hua-packages
firefox-93.2.2 found in github.com/MordragT/hua-packages
```

This command is an alias for

```bash
hua profiles current packages add <name>
```

#### Removal

This will search all installed packages in the current profile and generation under
`/hua/profiles/current/generation/` and asks which to remove. On completion a new
generation is created

```bash
hua remove firefox

Which package do you want to remove ? (abort with ctrl+c)
> firefox-95.0.1-pwiq2139q92s1m3rhefw310
```

This command is an alias for

```bash
hua profiles current packages remove <name>
```


#### Search

This will search all cookbooks for the package name and return a list of packages.

```bash
hua search <name>
```

This command is an alias for

```bash
hua cookbooks search <name>
```