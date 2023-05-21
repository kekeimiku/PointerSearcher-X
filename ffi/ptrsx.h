#include <stdint.h>

typedef struct FFIMap {
  uintptr_t start;
  uintptr_t end;
  const char *path;
} FFIMap;

typedef struct PtrsXDumper PtrsXDumper;

struct PtrsXDumper *ptrsx_dumper_init(int pid);

void ptrsx_dumper_free(PtrsXDumper *ptr);

int ptrsx_create_pointer_map(PtrsXDumper *ptr, const char *path);

typedef struct PtrsXScanner PtrsXScanner;

struct PtrsXScanner *ptrsx_scanner_init(const char *path);

void ptrsx_scanner_free(PtrsXScanner *ptr);

const struct FFIMap *ptrsx_get_select_page(PtrsXScanner *ptr, unsigned int *len);

int ptrsx_scan_pointer_path(PtrsXScanner *ptr,
                            const char *name,
                            const struct FFIMap *selected_regions,
                            uint32_t regions_len,
                            const char *output_file,
                            uint32_t depth,
                            uintptr_t target_addr,
                            uintptr_t offset_ahead,
                            uintptr_t offset_behind);

int last_error_length(void);

int last_error_message(char *buffer, int length);
