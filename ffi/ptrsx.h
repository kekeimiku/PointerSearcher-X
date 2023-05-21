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
 * name: file name prefix; ignored when out is not null
 * selected_regions: C owned array of memory regions to scan
 * regions_len: length for the array above
 * output_file: C owned valid relative or absolute output path, pass NULL to
 * use default path ${name}.scandata
 * depth: max pointer scan depth. 7 is generally a good choice
 * SAFETY: Addr.path must not modified by C-Side
 */
void ptrsx_scan_pointer_path(const struct Addr *selected_regions,
                             uint32_t regions_len,
                             const char *output_file,
                             uint32_t depth,
                             uintptr_t target_addr,
                             uint32_t offset_ahead,
                             uint32_t offset_behind);

int last_error_length(void);

int last_error_message(char *buffer, int length);
