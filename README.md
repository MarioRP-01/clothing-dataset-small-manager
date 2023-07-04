# DataSet Manager

The objective of this project is to create a cli tool to manage the [clothing-dataset-small](https://github.com/alexeygrigorev/clothing-dataset-small) and insert it into a postgres database.

All the commands are strongly attached to the directory structure of the repository. So, if you want to use this tool, you should clone the repository and follow the instructions below.

## Pre-requisites

Before start, you should have **Rust** [installed](https://doc.rust-lang.org/book/ch01-01-installation.html) in your machine. Alternatively, you can create a docker container to compile the application inside it, although currently the repository does not have a Dockerfile to do that.

## Development

Execute the application with following command:

```shell
cargo run
```

If you want to pass arguments, it's possible to add them using `--`:

```shell
cargo run -- --help
```

## Release

To build the release version of the cli tool, run the following command:

```shell
cargo build --release
```

## More information

An actual use case explanation of this tool can be found in this [repository](https://github.com/MarioRP-01/app-apache-php).

## References

- [Random value from enum](https://stackoverflow.com/a/48491021)
- [OsString in Rust](https://doc.rust-lang.org/std/ffi/struct.OsString.html)
- [Use google distroless images](https://github.com/GoogleContainerTools/distroless/blob/main/examples/rust/Dockerfile)
