from ctypes import *
from typing import Optional, List, Tuple


class FFIModule(Structure):
    _fields_ = [
        ("_start", c_size_t),
        ("_end", c_size_t),
        ("_pathname", c_char_p),
    ]


class FFIRange(Structure):
    _fields_ = [
        ("_start", c_size_t),
        ("_end", c_size_t),
    ]

    def start(self, value: int):
        self._start = c_size_t(value)
        return self

    def end(self, value: int):
        self._end = c_size_t(value)
        return self


class FFIParam(Structure):
    _fields_ = [
        ("_addr", c_size_t),
        ("_depth", c_size_t),
        ("_srange", FFIRange),
        ("_lrange", POINTER(FFIRange)),
        ("_node", POINTER(c_size_t)),
        ("_last", POINTER(c_ssize_t)),
        ("_max", POINTER(c_size_t)),
        ("_cycle", c_bool),
        ("_raw1", c_bool),
        ("_raw2", c_bool),
        ("_raw3", c_bool),
    ]

    # target address
    def addr(self, value: int):
        self._addr = c_size_t(value)
        return self

    # scan maximum level
    def depth(self, value: int):
        self._depth = c_size_t(value)
        return self

    # scan range
    def srange(self, value: FFIRange):
        self._srange = value
        return self

    # perform a range scan of the last level alone
    def lrange(self, value: Optional[FFIRange]):
        if value is None:
            self._lrange = None
        else:
            self._lrange = pointer(value)
        return self

    # minimum pointer chain length
    def node(self, value: Optional[int]):
        if value is None:
            self._node = None
        else:
            self._node = pointer(c_size_t(value))
        return self

    # pointer chain must end at the specified offset
    def last(self, value: Optional[int]):
        if value is None:
            self._last = None
        else:
            self._last = pointer(c_ssize_t(value))
        return self

    # maximum number of results
    def max(self, value: Optional[int]):
        if value is None:
            self._max = None
        else:
            self._max = pointer(c_size_t(value))
        return self

    # compute circular references of pointer chain
    # for example:
    # P stands for offset->pointer
    # P1->P2->P3->P1->P5 will be shortened to P1->P5
    def cycle(self, value: bool):
        self._cycle = c_bool(value)
        return self


class PointerScan:

    LIBRARY_FUNCS = {
        "ptrscan_init": (POINTER(c_void_p),),
        "ptrscan_free": (None, POINTER(c_void_p)),
        "ptrscan_version": (c_char_p,),
        "ptrscan_attach_process": (c_int, POINTER(c_void_p), c_int),
        "ptrscan_list_modules": (
            c_int,
            POINTER(c_void_p),
            POINTER(POINTER(FFIModule)),
            POINTER(c_size_t),
        ),
        "ptrscan_list_modules_pince": (
            c_int,
            POINTER(c_void_p),
            POINTER(POINTER(FFIModule)),
            POINTER(c_size_t),
        ),
        "ptrscan_create_pointer_map": (
            c_int,
            POINTER(c_void_p),
            POINTER(FFIModule),
            c_size_t,
        ),
        "ptrscan_create_pointer_map_file": (
            c_int,
            POINTER(c_void_p),
            POINTER(FFIModule),
            c_size_t,
            c_char_p,
        ),
        "ptrscan_load_pointer_map_file": (c_int, POINTER(c_void_p), c_char_p),
        "ptrscan_scan_pointer_chain": (c_int, POINTER(c_void_p), FFIParam, c_char_p),
        "ptrscan_read_memory_exact": (
            c_int,
            POINTER(c_void_p),
            c_size_t,
            POINTER(c_uint8),
            c_size_t,
        ),
        "get_last_error": (
            c_char_p,
            c_int,
        ),
    }

    def _init_lib_functions(self):
        for k, v in self.LIBRARY_FUNCS.items():
            f = getattr(self._lib, k)
            f.restype = v[0]
            f.argtypes = v[1:]

    def __init__(self, libpath="libptrscan.dylib"):
        self._lib = cdll.LoadLibrary(libpath)
        self._init_lib_functions()
        self._ptr = self._lib.ptrscan_init()

    def __del__(self):
        self._lib.ptrscan_free(self._ptr)

    def _get_last_error(self, value: c_int) -> str:
        return self._lib.get_last_error(value).decode()

    def _check_error(self, value: c_int):
        if value < 0:
            err = self._get_last_error(value)
            raise Exception(err)

    # Get current version
    def version(self) -> str:
        return self._lib.ptrscan_version().decode()

    # Attach to process
    def attach_process(self, pid: int):
        ret = self._lib.ptrscan_attach_process(self._ptr, c_int(pid))
        self._check_error(ret)

    # Get a list of modules that can be used as static base addresses
    def list_modules(self) -> List[Tuple[int, int, str]]:
        modules_ptr = POINTER(FFIModule)()
        size = c_size_t()
        ret = self._lib.ptrscan_list_modules(self._ptr, byref(modules_ptr), byref(size))
        self._check_error(ret)
        modules = cast(modules_ptr, POINTER(FFIModule * size.value)).contents
        module_list = [
            (module._start, module._end, module._pathname.decode())
            for module in modules
        ]
        return module_list

        # Get a list of modules that can be used as static base addresses

    def list_modules_pince(self) -> List[Tuple[int, int, str]]:
        modules_ptr = POINTER(FFIModule)()
        size = c_size_t()
        ret = self._lib.ptrscan_list_modules_pince(
            self._ptr, byref(modules_ptr), byref(size)
        )
        self._check_error(ret)
        modules = cast(modules_ptr, POINTER(FFIModule * size.value)).contents
        module_list = [
            (module._start, module._end, module._pathname.decode())
            for module in modules
        ]
        return module_list

    # Create pointer data in memory
    # It is created based on the passed in basic module address range `module.start` and `module.end`.
    # `module.pathname` is a file path, for library users you should handle this as needed
    # `module.pathname`, in order to facilitate library users to resolve static addresses by themselves, the rules are set by users themselves.
    # For example, only pass in the file name instead of the entire path, use the index to process the same module name,
    # Scanning the pointer chain will program output the contents of the static base address part according to `module.name`.
    # If you know memory well, you can also pass in a specific address range as needed.
    # For example, merge consecutive areas with the same module name
    def create_pointer_map(self, modules: List[Tuple[int, int, str]]):
        modules_ptr = (FFIModule * len(modules))()
        for i, module_tuple in enumerate(modules):
            start, end, pathname = module_tuple
            modules_ptr[i]._start = start
            modules_ptr[i]._end = end
            modules_ptr[i]._pathname = pathname.encode()
        ret = self._lib.ptrscan_create_pointer_map(
            self._ptr, modules_ptr, len(modules_ptr)
        )
        self._check_error(ret)

    # Create pointer mapping in file
    # It is created based on the passed in basic module address range `module.start` and `module.end`.
    # `module.pathname` is a file path, for library users you should handle this as needed
    # `module.pathname`, in order to facilitate library users to resolve static addresses by themselves, the rules are set by users themselves.
    # For example, only pass in the file name instead of the entire path, use the index to process the same module name,
    # scanning the pointer chain will program output the contents of the static base address part according to `module.name`.
    # If you know memory well, you can also pass in a specific address range as needed.
    # For example, merge consecutive areas with the same module name
    def create_pointer_map_file(
        self, modules: List[Tuple[int, int, str]], pathname: str
    ):
        modules_ptr = (FFIModule * len(modules))()
        for i, module_tuple in enumerate(modules):
            start, end, name = module_tuple
            modules_ptr[i]._start = start
            modules_ptr[i]._end = end
            modules_ptr[i]._pathname = name.encode()
        ret = self._lib.ptrscan_create_pointer_map_file(
            self._ptr, modules_ptr, len(modules_ptr), pathname.encode()
        )
        self._check_error(ret)

    # Load pointer mapping file into memory
    def load_pointer_map_file(self, pathname: str):
        ret = self._lib.ptrscan_load_pointer_map_file(self._ptr, pathname.encode())
        self._check_error(ret)

    # Scan pointer chain
    # It is thread-safe. If you have multiple target address parameters, you can scan them in multiple threads at the same time.
    # regarding pointer chain format analysis, each item starts with `$module.name+$offset`
    # As a static base address, followed by the pointer chain offset, separated by `.`, the base address `offset` and subsequent
    # offsets are both decimal numbers
    def scan_pointer_chain(self, param: FFIParam, pathname: str):
        ret = self._lib.ptrscan_scan_pointer_chain(self._ptr, param, pathname.encode())
        self._check_error(ret)
