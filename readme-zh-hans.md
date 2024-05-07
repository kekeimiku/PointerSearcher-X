# PointerSearcher-X

> 内存中的指针链自动化扫描工具

## 平台支持:

- [x] aarch64-darwin

- [x] aarch64-apple-ios

- [x] x86_64-linux-gnu

- [x] aarch64-linux-gnu

- [x] aarch64-linux-android (beta)

- [ ] x86_64-darwin (alpha)

- [ ] x86_64-windows (alpha)

- [ ] nintendo-switch

## 如何使用 ?

本项目提供了一个简单的命令行程序示例，最简单的用法：

```shell
ptrscan scan_pointer_chain use_process --pid 123 --addr-list 0x1234567 --depth 4 --range 0:3000
```

更多高级用法请参考 `ptrscan --help`

强烈建议您使用 [libptrsx.h](https://github.com/kekeimiku/PointerSearcher-X/blob/main/libptrscan/libptrsx.h) 创建自己的指针扫描程序，以便获得最大程度的灵活性。

## 关于

它只是为了解决下面两个问题所创建的，不过现在已经扩展到其它平台。

https://github.com/scanmem/scanmem/issues/431

https://github.com/korcankaraokcu/PINCE/issues/15

妈的全网搜不到个支持 Linux/Mac 的指针扫描器，所以我编写了它，并且尽可能让它跨平台。

它使用 AGPL 协议开源，开源免费，商用需要付费，只是防君子不防小人。

## 免责声明

编写它只是为了学习rust，没有恶意目的。
