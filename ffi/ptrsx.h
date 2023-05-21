#include <stdint.h>

typedef struct PtrsX PtrsX;

typedef struct Addr {
  uintptr_t start;
  uintptr_t end;
  const char *path;
} Addr;

struct PtrsX *ptrsx_init(int pid);

void ptrsx_free(struct PtrsX *ptr);

int ptrsx_create_pointer_map(struct PtrsX *ptr, const char *path);

const struct Addr *ptrsx_load_pointer_map(struct PtrsX *ptr,
                                          const char *path,
                                          unsigned int *length);

/**
 * name: file name prefix, NULL-terminated C string; ignored when out is not
 * null selected_regions: borrowed array of memory regions to scan
 * regions_len: length for the array above
 * output_file: borrowed valid relative or absolute output path, pass NULL to
 *     use default path `${name}.scandata`; NULL-terminated C string
 *
 * for other arguments, check documents of
 * `ptrsx_scanner::cmd::SubCommandScan::perform`
 *
 * Errors:
 *     -1: ptr or name is NULL
 *     -2: ptrsx did not load a pointer map, or those map is already consumed
 *     -3: other rust-side errors, check error messages.
 * SAFETY: Addr.path must not modified by C-Side
 */
int ptrsx_scan_pointer_path(struct PtrsX *ptr,
                            const char *name,
                            const struct Addr *selected_regions,
                            uint32_t regions_len,
                            const char *output_file,
                            uint32_t depth,
                            uintptr_t target_addr,
                            uintptr_t offset_ahead,
                            uintptr_t offset_behind);

int last_error_length(void);

int last_error_message(char *buffer, int length);
