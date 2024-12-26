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

## Credits

[@vimaexd](https://github.com/vimaexd) for their blog post !