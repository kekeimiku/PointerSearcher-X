# tutorial

以下教程都是linux环境，但也适用于其它支持的平台

## 死亡细胞的例子

当前版本 (v33,2023-03-17 - dbebf7d6 - 15 - Steam)

```shell
ptrsx cpm -p (pid)
# 这会创建两个文件
deadcells.maps      deadcells.pointers
```

假设金币地址为 0x754db988。然后我们可以直接关闭游戏。

```shell
ptrsx cpp --target 0x754db988 --pf deadcells.pointers --mf deadcells.maps --offset 0:800 --depth 7
# 这里会提示选择哪些模块 例如 [0: deadcells] [1: steamclient.so] [2: libsteam_api.so] [3: steam.hdll] [4: libopenal.so.1] [5: openal.hdll] [6: libmbedcrypto.so.1] [7: libmbedx509.so.0] [8: libmbedtls.so.10] [9: ssl.hdll] [10: libsndio.so.6.1] [11: libSDL2-2.0.so.0] [12: sdl.hdll] [13: libuv.so.1] [14: uv.hdll] [15: libturbojpeg.so.0] [16: fmt.hdll] [17: ui.hdll] [18: libhl.so] ...
# 我们输入对应的数字 选择多个用空格分隔 这里我选择了 deadcells 和 libhl.so，对应数字是 0 和 18
# 执行完成后会输出一份文件
0x754db988.23
```

0x754db988.23 就是计算结果，使用下面的命令查看结果

```shell
ptrsx spp --rf 0x754db988.23 --mf deadcells.maps
# 输出
libhl.so+0x27c8c0->0->64->312->664->32->136->72
libhl.so+0x27c8c0->0->288->312->152->480->136->72
libhl.so+0x27c8c0->0->288->120->480->480->136->72
libhl.so+0x27c8c0->0->288->88->536->480->136->72
libhl.so+0x27c8c0->0->288->216->328->480->136->72
libhl.so+0x27c8c0->0->288->280->200->480->136->72
libhl.so+0x27c8c0->0->288->248->248->480->136->72
libhl.so+0x27c8c0->0->288->184->376->480->136->72
libhl.so+0x27c8c0->0->288->152->424->480->136->72
libhl.so+0x27c8c0->0->288->376->48->480->136->72
libhl.so+0x27c8c0->0->288->344->104->480->136->72
libhl.so+0x27c8c0->0->64->312->664->88->136->72
...
# 结果很多，，省略
```

结果太多了，我们难以找到真正的有用的那条。所以我们第二次打开游戏，重复前面的操作。

然后我们对比出两次扫描结果中都有的那行结果(这很简单，写一个程序把每一行作为HashSet然后intersection就行了)

打开游戏重新找到金币的地址，然后输入命令查看过滤出来的指针路径结果是不是和找到的地址一样

```shell
ptrsx spv --pid (pid) --path "libhl.so+0x27c8c0->0->288->88->536->480->136->72"
# 输出
0x778b2010
```

我们查看 0x778b2010 的内容确实是金币所在的地址！

然后我们就可以每次都通过这条路径来找到金币无需花费时间重新寻找新的地址，也可以把这条结果发给别的伙伴用。

PS: 金币附近很可能还有一些其它重要数据，可能就在-+xx之类的位置，那时我们直接把最后的72修改掉即可，留给大家测试了。

PS: 对比两次的结果是为了确保游戏每次运行金币都会在这条路径，如果你想 当然也可以对比更多次。

PS: 如果你不急，可以把文件放到服务器上面慢慢计算。写一个脚本算完了发个通知。

PS: 扫描成本是 O(NN*D) (D:Max Depth,N:Offset Num)，offset和depth最好都不要太高。(本示例中(D:7,O:+800)只需要不到10s，这对多数游戏基本已经足够)

PS: 这个程序也支持负offset，不过它不算特别常见，建议正offset无效的时候尝试。