# MixxxKit

A command line tool that makes managing [Mixxx 2.4](https://mixxx.org/) libraries easy.

* [Warning](#warning)
* [How To Use](#how-to-use)
* [Contributors](#contributors)

## Warning

**_!!! This tool is experimental and comes with no warranties. The loss of your hot cues, crates, analyzed tracks, and more is possible. !!!_**

To protect against this, please make a backup of your entire [Mixxx settings directory](https://manual.mixxx.org/2.4/en/chapters/appendix/settings_directory) before using.

- Windows: `%LOCALAPPDATA%\Mixxx`
- macOS: `~/Library/Containers/org.mixxx.mixxx/Data/Library/Application Support/Mixxx`
- Linux: `~/.mixxx/`

Alternatively, if you trust me and my coding, you can use the following command as a shorthand:

```sh
$ mixxxkit backup
```

## How To Use

You may download the standalone executable from our [releases page](https://github.com/mass8326/mixxx-merge/releases). Use the appropriate file for your operating system.

Running the executable will bring up an interactive prompt in your terminal:

```
? What would you like to do?
> Backup
  Import
  Merge
[↑↓ to move, enter to select, type to filter]
```

You can pass the executable arguments to skip the prompting and enable use in scripts:

```sh
$ mixxxkit merge --help
$ mixxxkit merge source.db --force --debug mixxxkit::database
```

## Contributors

For Ubuntu:

```sh
$ sudo apt install -y musl-tools
```
