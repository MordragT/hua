# Profiles

In Hua every real user can create a hua profile. This profile keeps track of all the generations
created by the user.

## Commands

#### Add

This creates a profile for the current or specified user.
This operation can only be executed by a superuser or the current user self.

```bash
hua profiles add <name>?
```

#### Remove

This removes the profile for the current or specified user.
Be cautios this also deletes all generations attached to the profile.
This operation can only be executed by a superuser or the current user self.

```bash
hua profiles remove <name>?
```

### Generations

#### List

When you want to list all generations of another than the current user.

```bash
hua profiles <name> generations list
```

#### Remove

When you want to delete generations for another than the current user. You have to be
super-user to take this action.

```bash
hua profiles <name> generations remove  <old(all except the current) | (number w(weeks) | d(days) | h(hours))>
```

### Packages

#### Add

This adds a package to the specified user. You have to be a super-user or the current user to
execute this command.

```bash
hua profiles <name> packages add <package>

Which package do you want to install ? (abort with ctrl+c)
> firefox-95.0.1 found in github.com/MordragT/hua-packages
firefox-93.2.2 found in github.com/MordragT/hua-packages
```

#### Remove

This removes a pacakge of the specified user. You have to be a super-user or the current user to
execute this command.

```bash
hua profiles <name> packages remove <package>

Which package do you want to remove ? (abort with ctrl+c)
> firefox-95.0.1-pwiq2139q92s1m3rhefw310
```

## Roc

A profile can also be created by the system wide configuration file.

```elm
app "config"
    packages { pf: "hua" }
    imports [ pf.Profile ]
    provides [ profiles ] to pf

profiles : List Profile
profiles =
    tom = { name: "tom" }
    lisa = { name: "lisa" }
    [ tom, lisa ] 
```