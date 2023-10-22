#include <stdbool.h>
#include <stddef.h>

typedef struct PointerSearcherX PointerSearcherX;

#if defined(__linux__)
typedef int Pid;
#elif defined(__WIN32__)
typedef unsigned int Pid;
#elif defined(__APPLE__)
typedef int Pid;
#endif

typedef struct Module {
  size_t start;
  size_t end;
  char *name;
} Module;

typedef struct ModuleList {
  size_t len;
  const struct Module *data;
} ModuleList;

typedef struct AddressList {
  size_t len;
  const size_t *data;  
} AddressList;

typedef struct Params {
  size_t target;
  size_t depth;
  size_t node;
  size_t rangel;
  size_t ranger;
  const char *file_name;
} Params;

const char *get_last_error(struct PointerSearcherX *ptr);

struct PointerSearcherX *ptrsx_init(void);

void ptrsx_free(struct PointerSearcherX *ptr);

int create_pointer_map_file(struct PointerSearcherX *ptr, Pid pid, bool align,
                            const char *file_name);

int create_pointer_map(struct PointerSearcherX *ptr, Pid pid, bool align);

int load_pointer_map_file(struct PointerSearcherX *ptr, char *file_name);

struct ModuleList get_modules(struct PointerSearcherX *ptr);

int scanner_pointer_chain_with_module(struct PointerSearcherX *ptr,
                                      struct Module module,
                                      struct Params params);

int scanner_pointer_chain_with_address(struct PointerSearcherX *ptr,
                                       AddressList list, struct Params params);
