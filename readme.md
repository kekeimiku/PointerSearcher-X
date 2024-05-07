# PointerSearcher-X

> Memory dynamic pointer chain search tool.

([中文](./readme-zh-hans.md) / [English](./readme.md))

## Support:

- [x] aarch64-darwin

- [x] aarch64-apple-ios

- [x] x86_64-linux-gnu

- [x] aarch64-linux-gnu

- [x] aarch64-linux-android (beta)

- [ ] x86_64-darwin (alpha)

- [ ] x86_64-windows (alpha)

- [ ] nintendo-switch

## How to use ?

This project provides a simple command line program example, the simplest usage:

```shell
ptrscan scan_pointer_chain use_process --pid 123 --addr-list 0x1234567 --depth 4 --range 0:3000
```

For more advanced usage, please refer to `ptrscan --help`

It is strongly recommended that you create your own pointer scanner using [libptrsx.h](https://github.com/kekeimiku/PointerSearcher-X/blob/main/libptrscan/libptrsx.h) for maximum flexibility.

## About

It was only intended to solve the following two problems, but has now been extended to other platforms.

https://github.com/scanmem/scanmem/issues/431

https://github.com/korcankaraokcu/PINCE/issues/15

## Disclaimer

This is just for learning rust, no malicious purpose.
