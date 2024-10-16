# MixxxKit

> [!CAUTION]
> Cue points might not save after using this tool. Please [make a backup](https://manual.mixxx.org/2.4/en/chapters/appendix/settings_directory#location) and only use this tool for temporary or experimental uses.
>
> A potential reason for this issue is here: https://github.com/mixxxdj/mixxx/issues/12328

A command line tool that makes managing [Mixxx 2.4](https://mixxx.org/) libraries easy.

* [How To Use](#how-to-use)
* [Contributors](#contributors)

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

You may also pass the executable arguments to skip the prompting for use in scripts:

```sh
$ mixxxkit merge --help
$ mixxxkit merge source.db --force --debug mixxxkit::database
```

## Contributors

Install required dependencies for Ubuntu:

```sh
$ sudo apt install -y musl-tools
```
