```shell
ptrsx --help
Usage: ptrsx <command> [<args>]

Top-level command.

Options:
  --help            display usage information

Commands:
  cpm               Create Pointer Map.
  cpp               Calculate Pointer Path.
  spp               View `CPP` result file
  spv               Get the address pointed to by the pointer path.
```


Create Pointer Map

```shell
ptrsx cpm -p pid
# result
xxx.maps      xxx.pointers
```

Calculate Pointer Path

```shell
ptrsx cpp --target 0x600002b10000 --pf hello.pointers --mf hello.maps --offset 0:800 --depth 7
# result
0x600002b10000.23
```

View `cpp` result file

```shell
ptrsx spp --rf 0x600002b10000.23 --mf xxx.maps
# result
libxxx.dylib+0x11c028->0->0->16->16->16->16->0
libxxx.dylib+0x11c028->0->0->16->16->16->0
libxxx.dylib+0x11c028->0->0->16->16->0
libxxx.dylib+0x11c028->0->0->16->0
libxxx.dylib+0x11c028->0->0->0
...
```
