# 用途懂得都懂

只在m2芯片mbp上测试，应该适用于所有m芯片arm macos程序

前往 [release](https://github.com/kekeimiku/ups/releases) 页面下载此程序，前往 [changelog](https://github.com/kekeimiku/ups/blob/main/changelog.md)(重要) 查看更改日志。

```
Usage: ups -p <pid> -t <target> [-o <offset>] [-d <depth>] [-s <start>]

(macos) dynamic pointer path scanner.
version 0.0.4-beta
author: kk <kekelanact@gmail.com>

Options:
  -p, --pid         process id, type int32
  -t, --target      target address, type uint64-hex-list, use '-' to separate
                    multiple parameters
  -o, --offset      scan offset, default 0:64, type int16:int16
  -d, --depth       scan depth, default 7, max 11 , type uint8
  -s, --start       start from specified address, type uint64-hex-list, use '-'
                    to separate multiple parameters, for example:
                    0x111-0x222-0x333
  --help            display usage information

Example
./ups -p 24579 -t 0x600002da16e0

0x600002be0f28 + 0 > 0x6000022bc0f0 + 24 > 0x6000020dd660 + 16 > 0x6000020dd6a0 + -48 > 0x6000020dd6a0 + 64 > 600002da16e0
0x105705c90 + 0 > 0x6000022bc100 + 8 > 0x6000020dd660 + 16 > 0x6000020dd6a0 + -48 > 0x6000020dd6a0 + 64 > 600002da16e0
0x6000020dde28 + 0 > 0x6000022bc100 + 8 > 0x6000020dd660 + 16 > 0x6000020dd6a0 + -48 > 0x6000020dd6a0 + 64 > 600002da16e0
0x105705c90 + 0 > 0x600001460ae0 + 8 > 0x60000170c560 + 16 > 0x60000170c5a0 + 64 > 6000026ac5e0
... ...
```

如何使用扫描结果的例子: https://github.com/kekeimiku/dumpkey/blob/0bb96d7636e0e1480d2746a0476609843724e3af/src/main.rs#L8