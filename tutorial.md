#tutorial

The following tutorials are all in the linux environment, but are also applicable to other supported platforms

## Example of dead cells

Current Version (v33,2023-03-17 - dbebf7d6 - 15 - Steam)

```shell
ptrsx cpm -p (pid)
# This will create a directory
deadcells
```

Suppose the gold coin address is 0x754db988. Then we can close the game directly.

```shell
ptrsx cpp --target 0x754db988 --dir deadcells --offset 0:600 --depth 7
# Here you will be prompted which modules to choose, such as [0: deadcells] [1: steamclient.so] [2: libsteam_api.so] [3: steam.hdll] [4: libopenal.so.1] [5: openal.hdll] [6: libmbedcrypto.so.1] [7: libmbedx509.so.0] [8: libmbedtls.so.10] [9: ssl.hdll] [10: libsndio.so.6.1] [11: libSDL2-2.0. so.0] [12: sdl.hdll] [13: libuv.so.1] [14: uv.hdll] [15: libturbojpeg.so.0] [16: fmt.hdll] [17: ui.hdll] [18: libhl.so] ...
# We enter the corresponding number, select multiple and separate them with spaces Here I chose deadcells and libhl.so, the corresponding numbers are 0 and 18
# After the execution is complete, a directory will be output
0x754db988
```

0x754db988 stores the result, use the following command to view the result

```shell
ptrsx spp --dir 0x754db988
# result
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
```

There were so many results that it was difficult for us to find the one that was really useful. So we open the game for the second time and repeat the previous operation.

Then we compare the results of the row in the two scan results (this is very simple, write a program to use each row as a HashSet and then intersect it)

Open the game to find the address of the gold coin again, and then enter the command to check whether the result of the filtered pointer path is the same as the found address

```shell
ptrsx spv --pid (pid) --path "libhl.so+0x27c8c0->0->288->88->536->480->136->72"
# result
0x778b2010
```

We check that the content of 0x778b2010 is indeed the address where the gold coin is located!

Then in the future, we can find gold coins through this road every time without spending time looking for a new address, and we can also send this result to other brothers.

PS: There may be some important data in the gold coin attachment, which may be in the position of -+xx. At that time, we can directly modify the last 72, and leave it to everyone to test.

PS: The result of comparing twice is to ensure that the gold coins will be on this road every time the game is run, and you can compare more times if you want.

PS: If you are not in a hurry, you can put the file on the server and calculate slowly. After writing a script, send out a general knowledge.

PS: The scanning cost is O(NN*D) (D:Max Depth,N:Offset Num), and the offset and depth should not be too high. (In this example (D:7,O:+600) only needs to be less than 10s, which may be enough for many programs)

PS: This program also supports negative offset, but it is not very common. It is recommended to try it when the correct offset is invalid.