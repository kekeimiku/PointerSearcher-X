# 用途懂得都懂

只在m2芯片mbp上测试，应该适用于所有m芯片arm macos程序
```
./ups --help
Usage: ups [OPTIONS]

Options:
  -p, --pid <pid>        process id, type int32
  -t, --target <target>  target address, type uint64-hex
  -d, --depth <depth>    scan depth, default 7, type uint8
  -o, --offset <offset>  scan offset, default -128:128, type int16:int16
  -h, --help             Print help
  -V, --version          Print version

Example
./ups -p 24579 -t 0x600002da16e0

0x600002be0f28 + 0 > 0x6000022bc0f0 + 24 > 0x6000020dd660 + 16 > 0x6000020dd6a0 + -48 > 0x6000020dd6a0 + 64 > 600002da16e0
0x105705c90 + 0 > 0x6000022bc100 + 8 > 0x6000020dd660 + 16 > 0x6000020dd6a0 + -48 > 0x6000020dd6a0 + 64 > 600002da16e0
0x6000020dde28 + 0 > 0x6000022bc100 + 8 > 0x6000020dd660 + 16 > 0x6000020dd6a0 + -48 > 0x6000020dd6a0 + 64 > 600002da16e0
...
```

# todo
- [ ] 优化命令行解析
- [ ] 可选多线程支持
- [ ] 支持直接扫描dump文件（这代表应该可以扫描大多数平台，例如pc和游戏机）
- [ ] 过滤一堆垃圾结果（这需要一个好用的arm64反汇编库，暂时rust没有）