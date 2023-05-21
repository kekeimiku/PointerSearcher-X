#include "ptrsx.h"
#include <stdint.h>
#include <stdio.h>

int main() {
  int pid;
  printf("input pid: ");
  scanf("%d", &pid);

  PtrsXDumper *const ptrsx_dumper = ptrsx_dumper_init(pid);
  if (!ptrsx_dumper) {
    return -1;
  }

  ptrsx_create_pointer_map(ptrsx_dumper, "test.map");
  ptrsx_dumper_free(ptrsx_dumper);

  PtrsXScanner *ptrsx_scanner = ptrsx_scanner_init("test.map");
  if (!ptrsx_scanner) {
    return -1;
  }

  unsigned int region_size;
  const FFIMap *maps = ptrsx_get_select_page(ptrsx_scanner, &region_size);

  uintptr_t target_addr;
  printf("input target address: ");
  scanf("%jx", &target_addr);
  printf("got target addr: %jx\n", target_addr);

  int status = ptrsx_scan_pointer_path(ptrsx_scanner, "test", maps, region_size,
                                       NULL, 7, target_addr, 256, 256);
  if (status != 0) {
    int error_len = last_error_length();
    char error_msg[error_len];
    last_error_message(error_msg, error_len);
    printf("error occured in scanning: %s\n", error_msg);
  }
  ptrsx_scanner_free(ptrsx_scanner);
}
