## 这是一个简易的教程 WIP

以微信举例(AppStore WeChat 3.7.0 (25070))，同样适用于其它程序

第一次我们用某种方法找到了想要的地址为 `0x6000027416e0` 。

```shell
ups-cli -p $(pgrep WeChat |head -1) -t 0x6000027416e0 -d 11
```
输出 `WeChat-6000027416e0.txt` 。

第二次我们用某种方法找到了想要的地址为 `0x600001a2cac0` 。

```shell
ups-cli -p $(pgrep WeChat |head -1) -t 0x600001a2cac0 -d 11
```
输出 `WeChat-600001a2cac0.txt` 。

对比两次扫描结果

```shell
ups-diff WeChat-600001a2cac0.txt WeChat-6000027416e0.txt
```
输出
```shell
WeChat+0x5327c90->0->8->24->16->32->8->8->64->8->0->0
WeChat+0x5327c90->0->8->8->16->32->8->8->64->8->0->0
```

验证结果

```rust
// open process
    let pid = env::args()
        .nth(1)
        .ok_or(Error::Other("args error"))?
        .parse()
        .map_err(|_| Error::Other("parse pid error"))?;
    let proc = Process::open(pid)?;

    // WeChat+0x5327c90->0->8->24->16->32->8->8->64->8->0->0
    // address = WeChat+0x5327c90
    let mut address = proc
        .get_maps()
        .find(|m| m.pathname().ends_with("WeChat"))
        .map(|n| n.start() + 0x5327c90)
        .ok_or(Error::Other("find module error"))?;

    // 8字节的指针
    let mut buf = vec![0; 8];

    // 最后少一个数字 0
    // 路径: 0->8->24->16->32->8->8->64->8->0->0
    // 需要: 0->8->24->16->32->8->8->64->8->0
    for off in "0->8->24->16->32->8->8->64->8->0"
        .split("->")
        .map(|off| off.parse::<usize>())
    {
        let off = off.map_err(|_| Error::Other("parse off error"))?;
        proc.read_at(address + off, &mut buf)?;
        address = bytes_to_usize(buf.as_mut_slice())?;
    }

    // 这就是目标地址
    println!("{address:#x}");
```

如果结果过多可以运行第三次扫描，然后用第一次和第二次`diff`的结果和第三次对比，以此类推。

如果结果不正确，可以增加 `offset (-o)` 以及 `depth (-d)` 然后继续上述步骤。

## TL;DR

如果遇到提示 Vmmap(OpenProcess(5)) 错误，就是权限不足。