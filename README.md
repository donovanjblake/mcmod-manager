# mcmod-manager

A mod manager for Minecraft written in Python that uses the Modrinth Labrinth API.

<!-- TOC -->
- [Quick Start](#quick-start)
- [Arguments](#arguments)
- [TOML Format](#toml-format)
    - [Example](#example)
<!-- /TOC -->

## Quick Start

- Create a mcmod.toml, example:

  ```toml
  [defaults]
  game_version = "1.21.5"
  loader = "fabric"

  [projects]
  sodium.defaults = true
  faithful-x64 = {defaults = true, loader = "minecraft"}
  ```

- Install this project with your choice of pip, uv, etc
- Run `mcmod --config mcmod.toml --download mods-1.21.5`

## Arguments

`--config CONFIG`

Give a path to a mod list toml file to load. If omitted, try to load from `./mcmod.toml`.

`--download FOLDER`

Give a path to download the found mods to. If the folder does not exist, create it. Does not delete
items in the folder, but will overwrite them.

`--validate`

Developer use. Validate that all internal enumerations are up to date.

## TOML Format

`defaults.game_version`

A string that represents the target version of Minecraft.

`defaults.loader`

The default mod loader to use.

`projects`

A dictionary of the projects to download. For more details, see `projects-item`.

`projects-item.defaults`

Use the given defaults, e.g. `defaults.game_version`. If other members are specified, they will
override the defaults.

`projects-item.game_version`

The target Minecraft version for this project. May be needed if a project still works, but does not
get updated.

`projects-item.loader`

The mod loader for this project.

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
