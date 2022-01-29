# grill

A Package Manager for the [Beef Programming Language](https://github.com/beefytech/Beef)

Browse packages and manage your own at https://grillpm.vercel.app/

# Getting started

Download the [latest release of the CLI](https://github.com/RogueMacro/grill/releases/latest).
Add a `Package.toml` manifest file in the root directory of your project.
Here is an example manifest:

```toml
[package]
name = "foo"
version = "0.1.0"

[dependencies]
bar = "1.3.2"
```

After adding `Package.toml`, run `grill make` to build a workspace.

**Note:** Any projects added manually to a workspace will be removed when running `grill make`, unless they are specified in `Package.toml`. Git URLs, BeefLibs and relative dependencies are not supported yet.

# Installing a package to BeefLibs

You can install packages (or repositories) into the `BeefLibs` folder by using `grill install <package>` or `grill install --git <url>`. The library can then be added to workspaces in the IDE.

**Note:** BeefLibs are not supported in packages (running `grill make` will remove those libraries from the workspace).

# Publishing packages

To publish packages, you need to get your API token on the website at Account > Settings > Authorization.
Run `grill login` and paste your token there.

After logging in through the CLI, make sure you commit and push your changes, then run `grill publish` and confirm the version and commit. It will not succeed if the commit isn't found remotely.
