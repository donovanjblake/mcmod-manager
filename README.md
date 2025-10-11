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

`-v, --game-version <GAME_VERSION>`

Override the default game version in the config.

`-l, --loader <LOADER>`

Override the default mod loader in the config.

`-d, --download FOLDER`

Download the files to the given directory. Will not delete files already in the directory, but will
overwrite files in the directory.

`--install`

Install the project files into their appropriate directories under the `.minecraft` folder. Any
existing mods, resource packs, or data packs will be deleted.

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

`paths`

`table`: Optional. A dictionary of path overrides for the program. See the table below for the
default paths used by the program.

| Path            | Windows                             | MacOS                                      | Linux                              |
| :-------------- | :---------------------------------- | :----------------------------------------- | :--------------------------------- |
| `<HOME>`        | `C:\Users\<USER>`                   | `/Users/<USER>`                            | `/home/<USER>`                     |
| `data`          | `<HOME>\AppData\Local\mcmod`        | `<HOME>/Library/Application Support/mcmod` | `/home/<USER>/.local/shares/mcmod` |
| `dot_minecraft` | `<HOME>\AppData\Roaming\.minecraft` | `<HOME>/.minecraft`                        | `/home/<USER>/.minecraft`          |
| `temp`          | `C:\Temp\mcmod`                     | `confstr(_CS_DARWIN_USER_TEMP_DIR,…)`      | `/tmp/mcmod`                       |

`paths.data`

`string`: Optional. The path to the program's data directory. _This path is currently not used._

`paths.dot_minecraft`

`string`: Optional. The path to the .minecraft directory.

`paths.temp`

`string`: Optional. The path to the program's temp directory.

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

[paths]
data = "/home/alice/mcmod-data"

[projects]
sodium.defaults = true
faithful-32x = {defaults = true, loader = "minecraft"}

[optional-projects]
distanthorizons.defaults=true
```

In this example, the target version of minecraft is 1.21.5, and the preferred mod loader is Fabric.

The path to the programs data _(currently not used)_ is overriden from the default application data
path to `/home/alice/mcmod-data`.

The projects to download are sodium, faithful-32x, and distanthorizons.

The version of Sodium to be downloaded will be the latest version that supports Minecraft 1.21.5 and
uses the Fabric mod loader. If this is not available, the program will fail.

The version of Faithful to be downloaded will be the latest version that supports Minecraft 1.21.5
and uses the normal minecraft loader, because it is a resource pack, not a mod. If this is not
available, the program will fail.

The version of Distant Horizons to be downloaded wil lbe the latest version that supports Minecraft
1.21.5 and uses the Fabric mod loader. If this is not available, the program will skip it and
continue.
