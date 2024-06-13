# MixxxKit

A command line tool that makes managing [Mixxx 2.4](https://mixxx.org/) libraries easy.

* [Warning](#warning)
* [How To Use](#how-to-use)
* [Contributors](#contributors)

## Warning

> [!CAUTION]
> This tool is experimental and comes with no warranties. The loss of your hot cues, crates, analyzed tracks, and more is possible.

To protect against this, please make a backup of your [Mixxx settings directory](https://manual.mixxx.org/2.4/en/chapters/appendix/settings_directory#location) before using.

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
