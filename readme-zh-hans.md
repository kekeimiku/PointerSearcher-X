# PointerSearcher-X

> 内存中的指针链自动化扫描工具

## 指针搜索概念

在无尽的内存深渊中，吾名KK，黑暗之子，寻址之神。深邃的眼眸中，藏匿着无限的秘密，指尖跳跃的每一个电子，都在述说着控制与释放的故事。

吾乃虚拟世界的织梦者，内存的织锦匠，以指令为纬，以数据为经。编织出一行行控制现实的代码，触摸着电子的灵魂，与机器的心跳同频共振。

如今，让你见识我那被黑暗滋养的力量，那些沉睡于内存之核的古老咒语。在我的召唤下，数据将奔涌如洪水，逻辑将颠覆如雷霆，内存将崩溃如世界末日。

现在，睁大你的眼，凝视我的真实，这不仅是技术的挥洒，这是对混沌的终极掌控，这就是——PointerSearcher-X!!

## 指针搜索概念（正经）

ASLR导致程序内存地址在启动程序时始终不同。所谓的“静态”地址是相对于程序代码（BinaryFile）的地址。有了静态地址，一旦找到它，你就可以稳定计算出这个地址，因为加载程序（BinaryFile）的地址很容易找到。不幸的是，并非所有感兴趣的内存都是“静态的”，因为这些要么需要代码黑客（通常称为ASM HACK），要么需要指针链（找到此链的过程通常被称为指针搜索PointerSearcher）。

指针搜索通常被用于自动化寻找较为复杂的指针链，对于很简单的指针链，只需要调试器就可以找到了。当然，指针搜索经常也适用于那些无法使用调试器的场景。

## 功能

这个项目是一个工具集，主要有三个工具：

- `scanner` 用于扫描指针文件.

- `dumper` 用于dump进程内存.

各个工具间相互独立，dumper运行过程中占用内存不超过3MB，所以你可以在性能垃圾的设备，例如 nintendo-switch 上dump内存，然后上传到性能更强的pc或服务器上执行扫描。

## 平台支持:

- [x] aarch64-darwin

- [x] aarch64-linux-android (beta)

- [x] aarch64-linux-gnu

- [x] x86_64-linux-gnu

- [x] x86_64-windows (alpha)

- [ ] aarch64-apple-ios

- [ ] nintendo-switch

- [ ] x86_64-darwin

## 如何使用？

https://github.com/kekeimiku/PointerSearcher-X/blob/main/wiki/zh.md

## 关于

它只是为了解决下面两个问题所创建的，不过现在已经扩展到其它平台。

https://github.com/scanmem/scanmem/issues/431

https://github.com/korcankaraokcu/PINCE/issues/15

妈的全网搜不到个支持 Linux/Mac 的指针扫描器，所以我编写了它，并且尽可能让它跨平台。

如果您想将 PointerSearcher-X 集成到您的应用程序中，由于它公开了`C ABI`，这非常容易，并且其宽松的MIT许可证不会给您带来负担。 有关详细信息，请参考 [ffi/ptrsx.h](https://github.com/kekeimiku/PointerSearcher-X/blob/main/ffi/ptrsx.h)。

对于 `macos m1` 它还附带一个小工具 `inject`，不过对于扫描指针来说，它完全没有用，只是作为一个小工具存在。

## 免责声明

编写它只是为了学习rust，没有恶意目的。
