# Autobuild tools

LCCC Autobuild supports a number of tools via subcommands. 

All tools support the `--version` and `--help` options to print the autobuild version information, and the tool's help. 
Either option will cause an immediate exit of the tool

Running `autobuild` as `autobuild <tool> [tool args...]` will run the tool named *tool* with the specified arguments *tool args...*.

## config

Usage: `autobuild config [options...] [--] [target-dir]`

The config tool (also can be written as `configure`) generates an autobuild configuration cache from an autobuild project manifest.

See [config](config.md)

## build

Usage: `autobuild build [options...] [--] [build-dir]`

The build tool builds an autobuild project from its configuration. It can additionally do some partial configuration before the build.