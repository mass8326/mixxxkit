# MixxxKit

A command line tool that makes managing [Mixxx 2.4](https://mixxx.org/) libraries easy.

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

## Using

You may download and run a standalone executable from our [releases page](https://github.com/mass8326/mixxx-merge/releases). Use the appropriate file for your operating system.

Alternatively, you can run directly from source code if you have [Rust/Cargo](https://rustup.rs/) installed. Download or clone the repository and use `cargo run` in your terminal.

## Contributing

For Ubuntu:

```sh
$ sudo apt install -y musl-tools
```
