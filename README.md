# frust
A find replacement with SQL-like syntax.

*Note:* frust has a *pre-alpha* status and has a lot of bugs, issues and missing features.
We are grateful for every opened issue, pull request or idea.
Additionally this project was written while learning a good part of the [Rust programming language](https://www.rust-lang.org/en-US/).
Hence the quality of code will hopefully improve while learning more about Rust.

For news see our [github.io page](https://tbehner.github.io/frust-blog/).

## About
The find command as found on all (?) unixy operating systems is awesome. The problem is, that the syntax is apparently hard to remember.
A proof of this are anti-patterns like `find . | grep 'some regular expression'` which are commonly seen (but seldom admitted).
To find the desired functionality in the man page of find takes to much time.
One night we got frustrated enough (hence the name), to write our own frust, with a query language which is easy to remember.

## Installation
### From Source
Clone the repository to a desired location and then compile the binary with
```
cargo build --release
```
and move the executable somewhere on $PATH, for example
```
cp ./target/release/frust $HOME/bin
```

### Pre-compiled binaries
Check the download section on the [github.io page](https://tbehner.github.io/frust-blog/) or the releases for precompiled binaries.

## Introduction
The basic syntax of frust follows a SQL-like query syntax
```
frust "[attributes] from [directories] where [filter expression] exec [command];"
```
The semicolon at the end of the query is optional and, if not present, gets appended automatically.
All parts of the query are optional too, e.g. the following queries are also possible
```
frust "where [filter expression];"
frust "[attributes] where [filter expression] exec [command];"
frust "exec [command];"
```
The missing parts are filled with defaults which resemble the behaviour of the original find command.

The attributes are a comma seperated list of the following currently supported attributes:
  * name (full path of a file)
  * basename (only the name of the file, without the path)
  * size (size of the file)
  * mtime (last modification time)
  * atime (last access time)
  * ctime (creation time)
  * inode
  * type (either file, directory or link)
  * mimetype

These are also the attributes which are supported for use in the filter expression.
The currently supported operators are: <, <=, ==, >=, > and ~.
How these operators work depends on the attribute they are used with.

The name attribute implements for the ~ operator a comparison with a regular expression.
For example
```
frust "name where name ~ '^\.git'"
```
returns all files, where the path starts with a '.git' such as '.git/' or '.gitignore'.
Take a look at the [documentation](https://doc.rust-lang.org/regex/regex/index.html) of the regex crate to see which regular expressions are supported.

The size attribute can be compared with filesizes written as [number][unit], with units b, k, M, G, T (bytes, kilo bytes, mega bytes, giga bytes and terra bytes respectively) supported currently.
Some examples:
```
frust "name where size < 600M"
```
prints all files which are smaller than 600MByte and
```
frust "name where size >= 1G"
```
prints all files which are larger or equal 1Gbyte.

The time attributes support absolute time, e.g. 2017-04-01 12:18, and relative time, e.g. 1h.
The time units for relative times are m, h, D, M, Y.
For example
```
frust "name where mtime < -1m"
```
prints all files which where modified less then a minute ago,
```
frust "name where mtime > 2017-01-01"
```
prints all files which where modified after the 1st of January 2017,
```
frust "name where mtime > 17:00"
```
prints all files which where modified after 17:00 (or 5pm) and
```
frust "name where mtime > 2017-01-01 17:00"
```
prints all files which where modified after 1st of January 2017 at 17:00 (or 5pm).

The command in the exec part is executed for each file, which passes the filter expression.
For using the attributes of the found file, frust uses the [liquid template engine](https://shopify.github.io/liquid/).
Some examples:
```
frust "where name '\.rar$' and mtime < -1D exec rm {{name}}"
```
deletes all rar-archives which are older than a day or
```
frust "where name '\.jpg$' exec cp {{name}} {{name | remove_first: '.' | prepend: /media/backup | append: '.backup'}}" 
```
moves all jpg to `/media/backup` and appends the file extension `backup`.

frust has a colored output and colors can be configured in '$HOME/.config/frust/config.toml' and an example for a configuration file is given in 'example_config.toml'.
For best results use a terminal emulator with truecolor support. To switch off all colors use the '--no-color' option.
