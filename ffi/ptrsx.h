#include <sys/types.h>

typedef struct PointerScanTool PointerScanTool;

typedef struct Param {
  size_t addr;
  size_t depth;
  size_t left;
  size_t right;
  bool use_module;
  bool use_cycle;
  size_t *node;
  size_t *max;
  ssize_t *last;
} Param;

struct PointerScanTool *ptrs_init(void);

void ptrs_free(struct PointerScanTool *ptr);

const char *get_last_error(void);

int ptrs_set_proc(struct PointerScanTool *ptr, int pid);

int ptrs_create_pointer_map(struct PointerScanTool *ptr, const char *info_path,
                            const char *bin_path);

int ptrs_load_pointer_map(struct PointerScanTool *ptr, const char *info_path,
                          const char *bin_path);

int ptrs_scan_pointer_chain(struct PointerScanTool *ptr, struct Param param,
                            const char *file_path);

int compare_two_file(const char *file1, const char *file2, const char *outfile);

int ptrs_get_chain_addr(struct PointerScanTool *ptr, const char *chain,
                        size_t *addr);

int ptrs_filter_invalid(struct PointerScanTool *ptr, const char *infile,
                        const char *outfile);

int ptrs_filter_value(struct PointerScanTool *ptr, const char *infile,
                      const char *outfile, const uint8_t *data, size_t size);

int refresh_modules_cache(struct PointerScanTool *ptr);

int ptrs_filter_addr(struct PointerScanTool *ptr, const char *infile,
                     const char *outfile, size_t addr);
