## v0.2.2

Bugfix: wrong dyld_image size, Injection failed on some programs, error Kern(2)

## v0.2.1

just added an injector.

## v0.2.0

The ptrsx-dumper test command can optionally use the --num/-n parameter to view the contents of the last few bytes of the path.

Example:

```
ptrsx-dumper test --pid $(pgrep WeChat |head -1) --path "WeChat+0x53af490->0->8->8->16->32->8->8->64->8->0->0" -n 32
result:
0x600001670680
1171dfc9af2040e***********************094e294ec58b806a76e5f5f448
```

## v0.1.1

fix macOS check region

fix merge_bases

## v0.1.0
first version