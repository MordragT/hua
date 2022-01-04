# Cookbooks

## Create youre own cookbook

A cookbook is simply a folder which contains a list of recipe folders where every
recipe folder is build the following way:

- recipe-name
    - version1
        - recipe.roc
        - recipe.lock
    - version2
        - recipe.roc
        - recipe.lock
- recipe-name2
    - ...

What requirements must be met in the `recipe.roc` will be explained later in the chapter
about creating recipes.

---

Cookbooks  are stored inside the `/hua/cookbooks/<name>` folder. This allows us
to download cookbooks from different sources which help us to install our packages.
The cookbooks currently in use by the generation are saved in it under 
`generations/current/config/cookbooks.lock`. As the information which cookbook was used
to install the packages is vital to reproduce the installation, each change to the currently
in use cookbooks will create a new generation.

## Commands

#### Add

This will add a cookbook. It will download the content of the cookbook and
extract it into the `/hua/cookbooks/` folder aswell as creating a new generation with an updated
`generations/current/config/cookbooks.lock` file.

```bash
hua cookbooks add <https | http | ssh>
```

#### Remove

This will search the existing cookbooks and give a choice what to remove.
If the cookbook is still in use by generations, the generations in use will be printed.
If the removal succeeds, a new generation is created without the cookbook.

```bash
hua cookbooks remove github.com

Which cookbook do you want to remove ? (abort with ctrl+c)
> github.com/MordragT/hua-packages-p12ijrnfll3ijf082neq234t5
```

#### Update

This will check if the cookbooks used in the current generation had any changes and if they
had, a updated generation is created.

```bash
hua cookbooks update
```

#### Search

This will search all cookbooks for the package name
and will return a list of packages.

```bash
hua cookbooks search firefox

firefox-95.0.1 found in github.com/MordragT/hua-packages
firefox-93.2.2 found in github.com/MordragT/hua-packages
```

## Roc

Cookbooks can also be added via the system wide config file. This allows the user to create a
local cookbook which can be used to install self developed or self packaged programs.

```elm
app "config"
    packages { pf: "hua" }
    imports [ pf.Cookbook, pf.importSSH ]
    provides [ cookbooks ] to pf

# TODO: check if it is really possible to import remote files
cookbooks : List Cookbook
cookbooks =
    { cookbook: free } = importSSH "git@github.com:MordragT/free-software-cookbook"
    { cookbook: local } = import "./cookbooks/local"
    [ free, local ] 
```