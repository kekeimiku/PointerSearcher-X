from ctypes import *;

class Param(Structure):
    _fields_ = [
        ("addr", c_size_t),
        ("depth", c_size_t),
        ("node", c_size_t),
        ("rangel", c_size_t),
        ("ranger", c_size_t)
    ]

class Module(Structure):
    _fields_ = [
        ("start", c_size_t),
        ("end", c_size_t),
        ("name", c_char_p)
    ]

class ModuleList(Structure):
    _fields_ = [
        ("size", c_size_t),
        ("data", POINTER(Module))
    ]

lib = cdll.LoadLibrary("libptrsx.dylib")

lib.ptrsx_init.argtypes = []
lib.ptrsx_init.restype = POINTER(c_void_p)

lib.ptrsx_free.argtypes = (POINTER(c_void_p),)
lib.ptrsx_free.restype = None

lib.create_pointer_map_file.argtypes = (POINTER(c_void_p), c_int, c_bool, c_char_p, c_char_p)
lib.create_pointer_map_file.restype = c_int

lib.load_pointer_map_file.argtypes = (POINTER(c_void_p), c_char_p, c_char_p)
lib.load_pointer_map_file.restype = c_int

lib.get_modules_info.argtypes = (POINTER(c_void_p),)
lib.get_modules_info.restype = ModuleList

lib.scanner_pointer_chain.argtypes = (POINTER(c_void_p), ModuleList, Param, c_char_p)
lib.scanner_pointer_chain.restype = c_int

lib.get_last_error.argtypes = (POINTER(c_void_p),)
lib.get_last_error.restype = c_char_p
