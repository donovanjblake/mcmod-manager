# mcmod-manager

A mod manager for Minecraft written in rust that uses the Modrinth Labrinth API.

<!-- TOC -->

- [Quick Start](#quick-start)
- [Arguments](#arguments)
- [TOML Format](#toml-format)
  - [Example](#example)
  <!-- /TOC -->

## Quick Start

- Create a mcmod.toml in the same directory as the executable. Example:

  ```toml
  [defaults]
  game_version = "1.21.5"
  loader = "fabric"

  [projects]
  sodium.defaults = true
  faithful-x64 = {defaults = true, loader = "minecraft"}
  ```
- Run `mcmod --install`

## Arguments

`[CONFIG]`

Give a path to a mod list toml file to load. If omitted, try to load from `./mcmod.toml`.

`--download FOLDER`

Give a path to download the found mods to. If the folder does not exist, create it. Does not delete
items in the folder, but will overwrite them.

`--install`

Download the project files into their appropriate directories under the `.minecraft` folder.
**NOTE:** This does not work with datapacks, as they have to be installed for each world.

`--validate`

Developer use. Validate that all internal enumerations are up to date.

## TOML Format

`defaults`

`table`: A dictionary of default values for project information.

`defaults.game_version`

`string`: A string that represents the target version of Minecraft.

`defaults.loader`

`string`: The default mod loader to use.

`defaults.dot_minecraft`

`string`: Optional. The path to the `.minecraft` directory.

`defaults.temp`

`string`: Optional. The path to the temporary directory to use for files.

`projects`

`table`: A dictionary of the projects to download.

`projects.[project-name]`

`table`: A dictionary of the information for a project.

`projects.[project-name].defaults`

`bool`: For ommited members, use the values from the `defaults` table.

`projects.[project-name].game_version`

`string`: The target Minecraft version for this project. May be needed if a project still works, but
does not get updated. If omitted, use value from `defaults.game_version`.

`projects.[project-name].loader`

`string`: The mod loader for this project. If omitted, use value from `defaults.loader`. For
resource packs, use `minecraft`.

### Example

```toml
[defaults]
game_version = "1.21.5"
loader = "fabric"

[projects]
sodium.defaults = true
faithful-32x = {defaults = true, loader = "minecraft"}
```

In this example, the target version of minecraft is 1.21.5, and the preferred mod loader is Fabric.

The projects to download are sodium and faithful-32x.

The version of Sodium to be downloaded will be the latest version that supports Minecraft 1.21.5 and
uses the Fabric mod loader.

The version of Faithful to be downloaded will be the latest version that supports Minecraft 1.21.5
and uses the normal minecraft loader, because it is a resource pack, not a mod.
