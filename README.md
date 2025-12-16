# Webfishing MacOS Installer

An installer for a native version of webfishing on macos

## Why??

Webfishing is a game I love, however, it is poorely running on Whisky for macOS due to some unknown reason.

While searching for some solutions I stumbled across a [blog article](https://mae.wtf/blog/28102024-webfishing-mac) made by [@vimaexd](https://github.com/vimaexd),
it worked well in singleplayer, however as of now the multiplayer no longer works and this is why I decided to make this installer after figuring out how to patch this issue.


## Prerequisite
- Root privileges
- Steam

## Installation

To run the app, you can double-click on it, doing so will put the build folder inside your home directory `/Users/[you]/build`

You can also run it from the command line and doing so will put the build folder inside the current working directory.

## Implemented Patches

- renaming `steam_id_remote` dictionnary key to `remote_steam_id` to fix network spam detection that resulted in timeouts
- prevent the game from crashing when saving the options by not setting any values to `OS.windowed_borderless` because setting a value to it crashes the game somehow

## How to make a mod?

As you can see in the `example_mods` folder, a mod has typically two folders and a single `manifest.json` file having the following structure:
```jsonc
{
  "name": "Ship Mod", // Mod name
  "author": "Estym", // Author
  
  "pck_info": { // (Optional)
    "directory": "pck", // Relative folder path to where the mod resources are
    "resource_prefix": "res://Mods/Ship/" // Resource path prefix for the mod resources
  },
  
  "patches": [ // Array of patches
    {
      "resource": "res://Scenes/Entities/Player/player.gdc", // Resource to patch
      "patch_file": "patch/player.patch" // relative file path to the patch file
    }
  ],
  
  "deps": [] // Dependencies for the mod (Optional)
}
```

### Notes:
- The patch files are made by using `$ git diff [original_file] [modded_file] > file.patch`

## Credits

[@vimaexd](https://github.com/vimaexd) for their blog post !

[Godot RE Tools](https://github.com/GDRETools/gdsdecomp) for the amazing tooling that allows me to patch the game !
