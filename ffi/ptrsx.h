#include <stdbool.h>
#include <stddef.h>

#if defined(__linux__)
typedef int Pid;
#elif defined(__WIN32__)
typedef unsigned int Pid;
#elif defined(__APPLE__)
typedef int Pid;
#endif

typedef struct PointerSearcherX PointerSearcherX;

typedef struct Module {
  size_t start;
  size_t end;
  char *name;
} Module;

typedef struct ModuleList {
  size_t len;
  const struct Module *data;
} ModuleList;

typedef struct Param {
  size_t addr;
  size_t depth;
  size_t node;
  size_t rangel;
  size_t ranger;
} Params;

const char *get_last_error(struct PointerSearcherX *ptr);

struct PointerSearcherX *ptrsx_init(void);

void ptrsx_free(struct PointerSearcherX *ptr);

int create_pointer_map_file(struct PointerSearcherX *ptr, Pid pid, bool align,
                            const char *info_file_path,
                            const char *bin_file_path);

int load_pointer_map_file(struct PointerSearcherX *ptr, const char *bin_path,
                          const char *info_path);

int scanner_pointer_chain(struct PointerSearcherX *ptr,
                          struct ModuleList modules, struct Param params,
                          const char *file_path);

struct ModuleList get_modules_info(struct PointerSearcherX *ptr);