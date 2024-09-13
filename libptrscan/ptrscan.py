"""
    This program is offered under a commercial and under the AGPL license.
    For commercial licensing, contact us at kk <kekelanact@gmail.com>.  For AGPL licensing, see below.

    AGPL licensing:
    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU Affero General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU Affero General Public License for more details.

    You should have received a copy of the GNU Affero General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
"""

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
        "ptrscan_set_process": (c_int, POINTER(c_void_p), c_int),
        "ptrscan_set_modules": (
            c_int,
            POINTER(c_void_p),
            POINTER(FFIModule),
            c_size_t,
        ),
        "ptrscan_set_pointer_offset_symbol": (c_int, POINTER(c_void_p), c_char_p),
        "ptrscan_set_base_offset_symbol": (c_int, POINTER(c_void_p), c_char_p),
        "ptrscan_set_bitness": (c_int, POINTER(c_void_p), c_uint32),
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
        ),
        "ptrscan_create_pointer_map_file": (
            c_int,
            POINTER(c_void_p),
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
    def set_process(self, pid: int):
        ret = self._lib.ptrscan_set_process(self._ptr, c_int(pid))
        self._check_error(ret)

    # set the modules to be scanned
    def set_modules(self, modules: List[Tuple[int, int, str]]):
        modules_ptr = (FFIModule * len(modules))()
        for i, module_tuple in enumerate(modules):
            start, end, pathname = module_tuple
            modules_ptr[i]._start = start
            modules_ptr[i]._end = end
            modules_ptr[i]._pathname = pathname.encode()
        ret = self._lib.ptrscan_set_modules(self._ptr, modules_ptr, len(modules_ptr))
        self._check_error(ret)

    # option default `.`
    def set_pointer_offset_symbol(self, symbol: str):
        ret = self._lib.ptrscan_set_pointer_offset_symbol(self._ptr, symbol.encode())
        self._check_error(ret)

    # option default `+`
    def set_base_offset_symbol(self, symbol: str):
        ret = self._lib.ptrscan_set_base_offset_symbol(self._ptr, symbol.encode())
        self._check_error(ret)

    def set_bitness(self, bitness: int):
        ret = self._lib.ptrscan_set_bitness(self._ptr, c_uint32(bitness))
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
    def create_pointer_map(self):
        ret = self._lib.ptrscan_create_pointer_map(self._ptr)
        self._check_error(ret)

    # Create pointer mapping in file
    def create_pointer_map_file(self, pathname: str):
        ret = self._lib.ptrscan_create_pointer_map_file(self._ptr, pathname.encode())
        self._check_error(ret)

    # Load pointer mapping file into memory
    def load_pointer_map_file(self, pathname: str):
        ret = self._lib.ptrscan_load_pointer_map_file(self._ptr, pathname.encode())
        self._check_error(ret)

    # Scan pointer chain
    def scan_pointer_chain(self, param: FFIParam, pathname: Optional[str]):
        if pathname is None:
            ret = self._lib.ptrscan_scan_pointer_chain(self._ptr, param, None)
            self._check_error(ret)
        else:
            ret = self._lib.ptrscan_scan_pointer_chain(
                self._ptr, param, pathname.encode()
            )
            self._check_error(ret)
