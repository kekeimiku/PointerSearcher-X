#include "ptrsx.h"
#include <stdbool.h>
#include <stdio.h>

int main() {

  // dump pointer file
  int pid = 80805;
  int r = dumper_to_file(pid, false, "./aabb.dump");
  if (r != 0) {
    const char *err = get_last_error();
    printf("%s\n", err);
    return -1;
  }

  Scanner *scanner = NULL;

  int ret = scanner_init_with_file("hello-80805.dump", &scanner);
  if (ret != 0) {
    const char *err = get_last_error();
    printf("%s\n", err);
    return -1;
  }

  // choose something from pages list as a base module.
  // pointer scan will only output pointer chains whose starting address is
  // within the range of the selected module.
  PageVec pages = scanner_get_pages(scanner);

  for (int i = 0; i < pages.len; i++) {
    printf("[%zx %zx %s]\n", pages.data[i].start, pages.data[i].end,
           pages.data[i].path);
  }

  char buf[512];

  size_t addr;
  printf("please input target address:\n");
  scanf("%s", buf);
  sscanf(buf, "%zx", &addr);

  size_t depth;
  printf("please input depth:\n");
  scanf("%s", buf);
  sscanf(buf, "%zd", &depth);

  // pointer scan will ignore pointer chain whose length is less than node
  size_t node;
  printf("please input node:\n");
  scanf("%s", buf);
  sscanf(buf, "%zd", &node);

  size_t offset_n;
  printf("please input negative offset:\n");
  scanf("%s", buf);
  sscanf(buf, "%zd", &offset_n);

  size_t offset_p;
  printf("please input positive offset:\n");
  scanf("%s", buf);
  sscanf(buf, "%zd", &offset_p);

  printf("please input out dir:\n");
  scanf("%s", buf);

  struct Params param = {addr, depth, node, offset_n, offset_p, buf};

  struct Page data = {pages.data[0].start, pages.data[0].end,
                      pages.data[0].path};
  struct PageVec input_pages = {1, &data};

  // C++
  // std::vector<Page> select_pages;
  // struct PageVec input_pages = {select_pages.size(), select_pages.data()};

  ret = scanner_pointer_chain(scanner, &input_pages, param);
  if (ret != 0) {
    const char *err = get_last_error();
    printf("%s\n", err);
    return -1;
  }

  scanner_free(scanner);
  clear_last_error();

  return 0;
}

// libptrsx.dylib