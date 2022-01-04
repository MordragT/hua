# Configuration

Hua allows the user to create a central configuration in the programming language Roc.
This configuration is responsible then for creating user profiles, fetching cookbooks and installing packages.
This has the advantage that users can create reproducible bundles of software, that they can use on
different machines.
But for not so tech-savy people this may be a burden, so all features of hua can also be used just with
the terminal (potentially create simple ui around it).
In the following chapters both possibilites will always be shown.

## Commands

#### Update

To incorparate the recent changes of youre configuration into the system run the following command:

```bash
hua config update <boot | switch> <path to config.roc and config.lock>
```

---

Note:
In the package creation we will most definitly restrict the usage of fetching remote resources over the web
and other stuff depending on the operation (building, installation). We do this because hua is a decentralized
package manager but installing packages is a very security sensitive field. As the local configuration is usually
written by the enduser itself (or he is expected to atleast review it), we may not need to create restrictions -> evaluate.