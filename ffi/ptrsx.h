typedef struct Scanner Scanner;

typedef struct FFIPAGE
{
  unsigned long long start;
  unsigned long long end;
  const char *path;
} FFIPAGE;

typedef struct FFIParams
{
  unsigned long long depth;
  unsigned long long rangel;
  unsigned long long ranger;
  unsigned long long target;
  const char *out_dir;
} FFIParams;

int ptrsx_dumper_init(int pid, const char *out_file);

int last_error_length(void);

int last_error_message(char *buffer, int length);

struct Scanner *scanner_init(const char *in_file);

void scanner_free(struct Scanner *ptr);

int scanner_get_pages_len(struct Scanner *ptr);

struct FFIPAGE *scanner_get_pages(struct Scanner *ptr);

int scanner_pointer_chain(struct Scanner *ptr,
                          const struct FFIPAGE *pages,
                          unsigned long long len,
                          struct FFIParams params);
