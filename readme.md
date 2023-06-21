# PointerSearcher-X

> Memory dynamic pointer chain (scan/backtrack/search) tool.

## PointerSearch concept

ASLR causes the program memory address to always be different when you start the program. The so-called "static" addresses are addresses relative to the beginning of the program code (BinaryFile). With a static address, once you find it, you're done, because the address of the loader (BinaryFile) is easy to find. Unfortunately, not all memory of interest are "static", for these either code hack(often referred to as ASM hack) is needed or a pointer chain is needed(to find this chain is often referred to as pointer search).

## Features:

It contains three tools: 

- `ptrsx-scan` for scanning pointer files.

- `ptrsx-dumper` for dump process memory. 

- `ptrsx-inject` for dynamic library injection.

This program does not require the running status of the target process. It only needs a dump file to perform pointer scanning. You can use the pointer scanning function on any supported platform.
For example, generate a dump file on macOS and then perform scanning on a Linux server.

## Support:

- [x] x86_64-linux

- [x] aarch64-darwin

- [ ] aarch64-android

- [ ] nintendo-switch

- [ ] x86_64-windows

Currently, it is only tested on 64-bit systems and 64-bit targets. Although it can be compiled to other architectures, it cannot run normally. Support for 32-bit targets and other operating systems is in progress.

## Tutorial

https://www.bilibili.com/video/BV1Hh411E7oW/

[tutorial.md](tutorial.md)

## Disclaimer

This is just for learning rust, no malicious purpose.