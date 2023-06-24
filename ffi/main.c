#include "ptrsx.h"
#include <stdio.h>

int main()
{
    Scanner *scanner = scanner_init("WeChat-35247.dump");
    if (!scanner)
    {
        printf("scanner_init error\n");
        return -1;
    }

    int size = scanner_get_pages_len(scanner);
    printf("size: %d\n", size);

    FFIPAGE *pages = scanner_get_pages(scanner);
    if (!pages)
    {
        printf("get_pages error\n");
        return -1;
    }

    for (int i = 0; i < size; i++)
    {
        printf("[%llx %llx %s]\n", pages[i].start, pages[i].end, pages[i].path);
    }

    return 0;
}

// libptrsx.dylib