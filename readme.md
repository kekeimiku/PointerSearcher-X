# Disclaimer

This is just for learning rust, no malicious purpose.

# Tutorial

在目标设备上运行

```shell
ptrsx-dumper disk --pid $(pid)
```

输出的文件建议拷贝到电脑上 

```shell
ptrsx-scanner -f xxx.dump --target 0x1234567 --offset 0:800 --depth 7
# out
out.txt
```

