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