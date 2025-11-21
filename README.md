[![Crate](https://img.shields.io/crates/v/dynlibs)](https://crates.io/crates/dynlibs)
[![Docs](https://docs.rs/dynlibs/badge.svg)](https://docs.rs/dynlibs)

A simple, cross-platform program to display dynamic libraries used by an executable

## Overview

I found it a pain when writing cross platform programs to figure out which dynamic libraries I was linking with. On macOS, there is `otool` and `objdump`, on linux, `ldd`, and on Windows, `dumpbin.exe`. Once I discovered the awesome `goblin` crate, I was surprised no one had created a simple program to just dump the dynamic libraries used by an executable, so I created one.

Additionally, I wanted a simple way in my CICD pipeline to ensure I didn't accidentally add any dynamic library requirements that I wasn't expecting, so I added the `--only` flag to allow validating that only the expected dynamic libraries are required.

> NOTE: There aren't many surprises here, but if you aren't aware, `ldd` looks up transitive dependencies, but this utility does not. This means this utility lists only the direct dependencies of an executable, which is consistent with the other utilities listed above, and makes this program consistent across executable types and platforms.

## Install

```shell
cargo install --locked dynlibs
```

# Features

* Simple to use
* Platform independent (tested on Windows, Linux, macOS)
* CICD: Can validate only the expected libraries are present
* Can be used as a library crate as well, if needed

## Display Examples

Linux:

```bash
$ dynlibs /bin/ls
Binary type: ELF

Dynamic libraries:
    libselinux.so.1
    libc.so.6
```

Windows:

```shell
PS dynlibs c:\windows\hh.exe
Binary type: PE

Dynamic libraries:
    ADVAPI32.dll
    KERNEL32.dll
    msvcrt.dll
```

Mac:

```bash
% dynlibs /bin/ls
Binary type: Mach-O (Fat)

Dynamic libraries:
    Index 0:
        /usr/lib/libutil.dylib
        /usr/lib/libncurses.5.4.dylib
        /usr/lib/libSystem.B.dylib
    Index 1:
        /usr/lib/libutil.dylib
        /usr/lib/libncurses.5.4.dylib
        /usr/lib/libSystem.B.dylib
```

# CICD Validation

Exit code 1 (output to stderr):

```bash
% dynlibs -o libutil -o libSystem /bin/ls
Some dynamic libraries did not match the provided regexes:
    Index 0:
        /usr/lib/libncurses.5.4.dylib
    Index 1:
        /usr/lib/libncurses.5.4.dylib
```

Exit code 0 (no output):

```bash
% dynlibs -o libutil -o libSystem -o libncurses /bin/ls
```

## Status

This is a very simple program, and besides some possible slight enhancements, it is more or less complete.

## Contributions

Contributions are welcome as long they align with my vision for this crate.

