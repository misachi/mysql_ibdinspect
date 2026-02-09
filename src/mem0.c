/*
custom zlib decompression memory alloc/dealloc routines
*/

#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>

#define UNUSED __attribute__((unused))

/* Holds the data chunk from which to allocate memory during decompression */
typedef struct heap_t
{
    uint32_t len; // total size
    uint32_t off; // offset; allocated upto here
    char *data;
} heap_t;

/* Create heap */
void *mem_heap_create(void *ptr, uint32_t sz) {
    heap_t *heap = malloc(sz);
    if (!heap) {
        printf("malloc: Could not allocate heap");
        return NULL;
    }

    heap->off = 0;
    heap->len = sz;
    heap->data = (char *)ptr;
    return heap;
}

/* Free heap */
void mem_heap_free(void *ptr) {
    free(ptr);
}

/** Allocate memory for zlib. */
void *page_zip_zalloc(void *opaque,   /*!< in/out: memory heap */
                             uint32_t items, /*!< in: number of items to allocate */
                             uint32_t size)  /*!< in: size of an item in bytes */
{
    heap_t *heap = (heap_t *)opaque;
    uint32_t alloc_size = items * size;
    char *mem_allocd = NULL;
    if ((heap->off + alloc_size) >= heap->len)
    {
        printf("Could not allocate memory");
        return NULL;
    }

    mem_allocd = heap->data + heap->off;
    heap->off += alloc_size;
    return (void *)mem_allocd;
    
}

/** Deallocate memory for zlib. */
void page_zip_free(void *opaque UNUSED,  /*!< in: memory heap */
                          void *address UNUSED) /*!< in: object to free */
{
}
