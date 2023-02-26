# 这是什么？

这是一个指针扫描器，帮助人们找到想要的地址路径以及破解aslr [tutorial](tutorial.md) 。

前往 [release](https://github.com/kekeimiku/ups/releases) 页面下载此程序。

早期开发中，使用方法随时可能有较大变化，可能 `--help` 更新不及时，建议关注 [changelog](changelog.md) 查看更改日志。

```
Usage: ups -p <pid> -t <target> [-o <offset>] [-d <depth>] [-s <start>]

(macos) dynamic pointer path scanner.
version 0.0.5
author: kk <kekelanact@gmail.com>

Options:
  -p, --pid         process id, type int32
  -t, --target      target address, type uint64-hex-list, use '-' to separate
                    multiple parameters
  -o, --offset      scan offset, default 0:128, type int16:int16
  -d, --depth       scan depth, default 7, max 11 , type uint8
  -s, --start       start from specified address, type uint64-hex-list, use '-'
                    to separate multiple parameters, for example:
                    0x111-0x222-0x333
  --help            display usage information
```

# 平台支持

- [x] Linux(仅64位)

- [x] Macos

- [ ] Windows

- [ ] dump (内存转储文件)

# TL;DR

本工具暂时闭源（防止 unsafe ptsd 小子乱叫）。