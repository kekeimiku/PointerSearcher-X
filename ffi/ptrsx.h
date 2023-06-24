typedef struct Scanner Scanner;

typedef struct FFIPAGE {
  unsigned long long start;
  unsigned long long end;
  const char *path;
} FFIPAGE;

int last_error_length(void);

int last_error_message(char *buffer, int length);

int ptrsx_dumper_init(int pid, const char *out_file);

struct Scanner *scanner_init(const char *in_file);

void scanner_free(struct Scanner *ptr);

int scanner_get_pages_len(struct Scanner *ptr);

struct FFIPAGE *scanner_get_pages(struct Scanner *ptr);
