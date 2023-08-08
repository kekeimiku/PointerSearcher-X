#include <stddef.h>

typedef struct Scanner Scanner;

typedef struct FFIPAGE {
  size_t start;
  size_t end;
  const char *path;
} FFIPAGE;

typedef struct FFIParams {
  size_t depth;
  size_t node;
  size_t rangel;
  size_t ranger;
  size_t target;
  const char *out_dir;
} FFIParams;

int ptrsx_dumper_init(int pid, const char *out_file);

const char *last_error_message(void);

struct Scanner *scanner_init(const char *in_file);

void scanner_free(struct Scanner *ptr);

int scanner_get_pages_len(struct Scanner *ptr);

struct FFIPAGE *scanner_get_pages(struct Scanner *ptr);

int scanner_pointer_chain(struct Scanner *ptr,
                          const struct FFIPAGE *pages,
                          size_t len,
                          struct FFIParams params);
