#include "ptrsx.h"
#include <stdio.h>

int main() {
  Scanner *scanner = scanner_init("hello-51256.dump");
  if (!scanner) {
    printf("scanner_init error\n");
    return -1;
  }

  int size = scanner_get_pages_len(scanner);
  printf("size: %d\n", size);

  FFIPAGE *pages = scanner_get_pages(scanner);
  if (!pages) {
    printf("get_pages error\n");
    return -1;
  }

  for (int i = 0; i < size; i++) {
    printf("[%zx %zx %s]\n", pages[i].start, pages[i].end, pages[i].path);
  }

  struct FFIParams param = {4, 3, 200, 200, 0x600001ef0060, "./"};

  scanner_pointer_chain(scanner, pages, 1, param);

  return 0;
}

// libptrsx.dylib