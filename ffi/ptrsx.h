typedef struct PtrsX PtrsX;

typedef struct Addr {
  const void *start;
  const void *end;
  const char *path;
} Addr;

struct PtrsX *ptrsx_init(int pid);

void ptrsx_free(struct PtrsX *ptr);

int ptrsx_create_pointer_map(struct PtrsX *ptr, const char *path);

const struct Addr *ptrsx_load_pointer_map(struct PtrsX *ptr,
                                          const char *path,
                                          unsigned int *length);

int last_error_length(void);

int last_error_message(char *buffer, int length);
