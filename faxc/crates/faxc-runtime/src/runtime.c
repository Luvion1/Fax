/*
 * Fax Runtime - GC-aware memory allocation
 * 
 * This provides the memory allocation functions that the Fax compiler
 * generates calls to. For now, this uses system malloc/free.
 * 
 * To integrate FGC:
 * 1. Build FGC as a C-compatible library
 * 2. Replace the implementations below with FGC calls
 */

#include <stdlib.h>
#include <string.h>
#include <stdio.h>

/* GC State */
static int gc_initialized = 0;

/* Initialize GC - called at program start */
int fax_gc_init(void) {
    if (gc_initialized) {
        return 1;  /* Already initialized */
    }
    
    /* Initialize GC here when FGC is integrated */
    gc_initialized = 1;
    
    return 1;
}

/* Allocate memory using GC */
void* fax_gc_alloc(size_t size) {
    if (!gc_initialized) {
        fax_gc_init();
    }
    
    void* ptr = malloc(size);
    if (ptr == NULL) {
        fprintf(stderr, "GC allocation failed: out of memory\n");
        return NULL;
    }
    
    return ptr;
}

/* Allocate zeroed memory using GC */
void* fax_gc_alloc_zeroed(size_t size) {
    if (!gc_initialized) {
        fax_gc_init();
    }
    
    void* ptr = calloc(1, size);
    if (ptr == NULL) {
        fprintf(stderr, "GC allocation failed: out of memory\n");
        return NULL;
    }
    
    return ptr;
}

/* Register a root pointer */
int fax_gc_register_root(void* ptr) {
    /* When FGC is integrated, this will register the pointer as a root */
    return 1;
}

/* Unregister a root pointer */
int fax_gc_unregister_root(void* ptr) {
    /* When FGC is integrated, this will unregister the root */
    return 1;
}

/* Trigger garbage collection */
void fax_gc_collect(void) {
    /* Placeholder - full GC will be integrated with FGC */
}

/* Trigger young generation collection */
void fax_gc_collect_young(void) {
    /* Placeholder - generational GC will be integrated with FGC */
}

/* Shutdown GC */
void fax_gc_shutdown(void) {
    /* Cleanup GC when program exits */
    gc_initialized = 0;
}
