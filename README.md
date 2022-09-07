# grill

A Package Manager for the [Beef Programming Language](https://github.com/beefytech/Beef)

Browse packages and manage your own at https://grillpm.vercel.app/

# Getting started

Download the latest release of the CLI from [GitHub Releases](https://github.com/RogueMacro/grill/releases/latest).

Run `grill new MyProject` or `grill init` for existing Beef Workspaces. After creating a project you can add dependencies and run `grill make` to build your workspace. Here is an example manifest:

```toml
[Package]
Name = "MyGui"
Version = "0.1.0"
Description = "A small GUI application"

[Dependencies]
OpenGL = "3.3"
```

**Note:** The workspace file is generated automatically by Grill. Changes to it will be reverted when building the workspace. Fields specified in project files will be preserved.

# Publishing packages

To publish packages, you need to get your API token on the website at Account > Settings > Authorization.
Run `grill login` and paste your token there.

After logging in through the CLI, make sure you commit and push your changes, then run `grill publish` and confirm the version and commit. It will not succeed if the commit isn't found remotely.

# Installing a package to BeefLibs

You can install packages (or repositories) into the `BeefLibs` folder by using `grill install <package>` or `grill install --git <url>`. The library can then be added to workspaces in the IDE.

**Note:** BeefLibs are not supported in packages (running `grill make` will remove those libraries from the workspace).
