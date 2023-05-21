#include "ptrsx.h"
#include <stdint.h>
#include <stdio.h>

int main() {
  int pid;
  scanf("%d", &pid);

  PtrsX *const ptrsx = ptrsx_init(pid);
  if (!ptrsx) {
    return -1;
  }

  // ptrsx_create_pointer_map(ptrsx, "test.map");
  unsigned int region_size;
  const Addr *addrs = ptrsx_load_pointer_map(ptrsx, "test.map", &region_size);

  if (!addrs) {
    printf("pointer map load failed!\n");
    return -1;
  } else {
    printf("pointer map loaded!\n");
  }

  uintptr_t target_addr;
  scanf("%ju", &target_addr);

  int status = ptrsx_scan_pointer_path(ptrsx, "test", addrs, region_size, NULL,
                                       3, target_addr, 0, 32);
  if (-3 == status) {
    int error_len = last_error_length();
    char error_msg[error_len];
    last_error_message(error_msg, error_len);
    printf("error occured in scanning: %s\n", error_msg);
  }
  ptrsx_free(ptrsx);
}
