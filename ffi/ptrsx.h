#include <stddef.h>
#include <stdbool.h>

typedef struct Scanner Scanner;

typedef struct Page {
  size_t start;
  size_t end;
  char *path;
} Page;

typedef struct PageVec {
  size_t len;
  const struct Page *data;
} PageVec;

typedef struct Params {
  size_t target;
  size_t depth;
  size_t node;
  size_t rangel;
  size_t ranger;
  const char *dir;
} Params;

int dumper_to_file(int pid, const char *path, bool align);

const char *get_last_error(void);

void clear_last_error(void);

int scanner_init_with_file(const char *path, struct Scanner **ptr);

void scanner_free(struct Scanner *ptr);

struct PageVec scanner_get_pages(const struct Scanner *ptr);

int scanner_pointer_chain(struct Scanner *ptr, const struct PageVec *pages, struct Params params);
