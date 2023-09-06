## v0.4.0

### scanner

It is now possible to scan a chain of pointers between two addresses.

Command line parameter changes, `s1` behaves the same as the previous scan, manually selects a base module, and then scans the pointer chain from the base module to the target.

`s2` requires manually specifying an address using `s/start` and can now scan the chain of pointers between any two addresses.

```text
Commands:
  s1                Scan mode 1, select some modules to set as base addresses.
  s2                Scan mode 2, set address as the base address.
  diff              Compare and get the intersecting parts of two .scandata files.
```

Change pointer chain depth judgment. In the past, the base address was incorrectly treated as a first-level pointer.

### dumper

Optionally use the `--align` parameter to force the pointer into unaligned mode, which may be useful for dumping certain emulator processes

## v0.3.1

Bugfix.

## v0.3.0

Add `--node/n` option to ignore short pointer chains, default is 3. 

and some performance improvements

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