# PointerSearcher-X

> Memory dynamic pointer chain (scan/backtrack/search) tool.

([中文](./readme-zh-hans.md) / [English](./readme.md))

## PointerSearch concept

ASLR causes the program memory address to always be different when you start the program. The so-called "static" addresses are addresses relative to the beginning of the program code (BinaryFile). With a static address, once you find it, you're done, because the address of the loader (BinaryFile) is easy to find. Unfortunately, not all memory of interest are "static", for these either code hack(often referred to as ASM hack) is needed or a pointer chain is needed(to find this chain is often referred to as pointer search).

Pointer search is usually used to automatically find more complex pointer chains. For very simple pointer chains, only a debugger is needed to find them. Of course, pointer searches are often also useful in scenarios where a debugger cannot be used.

## Features:

It contains three tools: 

- `scanner` for scanning pointer files.

- `dumper` for dump process memory. 

## Support:

- [x] aarch64-darwin

- [x] aarch64-linux-android (beta)

- [x] aarch64-linux-gnu

- [x] x86_64-linux-gnu

- [x] x86_64-windows (alpha)

- [ ] aarch64-apple-ios

- [ ] nintendo-switch

- [ ] x86_64-darwin

## How to use?

https://github.com/kekeimiku/PointerSearcher-X/blob/main/wiki/en.md

## About

It was only intended to solve the following two problems, but has now been extended to other platforms.

https://github.com/scanmem/scanmem/issues/431

https://github.com/korcankaraokcu/PINCE/issues/15

If you want to incorporate PointerSearcher-X into your application, it's very easy. Its permissive MIT-style license won't burden you. See the [C API](https://github.com/kekeimiku/PointerSearcher-X/blob/main/ffi/ptrsx.h) for details.

## Disclaimer

This is just for learning rust, no malicious purpose.