# grill

A Package Manager for the [Beef Programming Language](https://github.com/beefytech/Beef)

Browse packages and manage your own at https://grillpm.vercel.app/

Documentation: https://roguemacro.gitbook.io/grill

# Getting started

Download the latest (pre)release of the CLI from [GitHub Releases](https://github.com/RogueMacro/grill/releases). `V0` is just a temporary build of Grill, since Grill relies on itself to pull packages and make a workspace. To build Grill you can either download this prebuilt executable or clone the dependencies yourself. This version cannot be used for publishing packages yet. If you want to publish packages you can download the legacy `v0.2.3` version. Note that `v0.2.3` and below can only be used to publish packages, and will (probably) fail otherwise.

Run `grill new MyProject` to create a new package. Add your chosen dependencies and run `grill make` to build your workspace. Here is an example manifest:

```toml
[Package]
Name = "MyGui"
Version = "0.1.0"
Description = "A small GUI application"

[Dependencies]
OpenGL = "3.3"
```

**Note:** The workspace file is generated automatically by Grill. It will preserve all properties of workspace- and project files, except dependencies and preprocessor macros. Any changes to those two properties will be reverted by Grill.

# Publishing packages

> Only available currently in `v0.2.3`

To publish packages, you need to get your API token on the website at Account > Settings > Authorization.
Run `grill login` and paste your token there.

After logging in through the CLI, make sure you commit and push your changes, then run `grill publish` and confirm the version and commit. It will not succeed if the commit isn't found remotely.
