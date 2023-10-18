# PointerSearcher-X

> 内存中的指针链自动化扫描工具

([中文](./readme-zh-hans.md) / [English](./readme.md))

## 指针搜索概念

ASLR导致程序内存地址在启动程序时始终不同。所谓的“静态”地址是相对于程序代码（BinaryFile）的地址。有了静态地址，一旦找到它，你就可以稳定计算出这个地址，因为加载程序（BinaryFile）的地址很容易找到。不幸的是，并非所有感兴趣的内存都是“静态的”，因为这些要么需要代码黑客（通常称为ASM HACK），要么需要指针链（找到此链的过程通常被称为指针搜索PointerSearcher）。

指针搜索通常被用于自动化寻找较为复杂的指针链，对于很简单的指针链，只需要调试器就可以找到了。当然，指针搜索经常也适用于那些无法使用调试器的场景。

## 功能

这个项目是一个工具集，主要有三个工具：

- `scanner` 用于扫描指针文件.

- `dumper` 用于dump进程内存.

各个工具间相互独立，dumper运行过程中占用内存不超过3MB，所以你可以在性能垃圾的设备，例如 nintendo-switch 上dump内存，然后上传到性能更强的pc或服务器上执行扫描。

## 平台支持:

- [x] aarch64-darwin

- [ ] aarch64-android

- [ ] nintendo-switch

- [x] x86_64-linux

- [x] x86_64-windows (版本大于等于 Windows 7)

- [ ] x86_64-darwin

## 教程

https://www.bilibili.com/video/BV1Hh411E7oW/

[tutorial.md](tutorial.md)

## 关于

它只是为了解决下面两个问题所创建的，不过现在已经扩展到其它平台。

https://github.com/scanmem/scanmem/issues/431

https://github.com/korcankaraokcu/PINCE/issues/15

如果您想将 PointerSearcher-X 合并到您的应用程序中，这非常容易，其宽松的MIT许可证不会给您带来负担。 有关详细信息，请参阅 [API 参考](https://github.com/kekeimiku/PointerSearcher-X/blob/main/ffi/ptrsx.h)。

## FAQ

[FAQ](./faq-zh.md)

## 免责声明

编写它只是为了学习rust，没有恶意目的。
