---
title: FGC - Fax Garbage Collector
description: Understanding the Fax Garbage Collector
---

## Overview

FGC (Fax Garbage Collector) is a custom generational garbage collector designed for predictable memory management.

## Architecture

### Generational Design

FGC uses a generational approach with multiple regions:

```
┌──────────────┐
│   Nursery    │  ← New allocations (4MB)
├──────────────┤
│    Small     │  ← < 256 bytes
├──────────────┤
│   Medium     │  ← 256B - 4KB
├──────────────┤
│    Large     │  ← > 4KB
├──────────────┤
│  Tenured     │  ← Long-lived objects
└──────────────┘
```

### Key Features

- **Nursery**: 4MB region for new allocations
- **Small/Medium/Large pages**: Size-based segregation
- **Tenured**: Objects that survive multiple collections
- **Write Barrier**: Track references from old to young
- **Parallel Marking**: Multi-threaded garbage collection

## Memory Layout

### Object Header

Every heap object has a header:

```
┌──────────────┬──────────────┬──────────────┐
│    Type      │    Color     │     Age      │
│   (8 bits)   │   (8 bits)   │   (8 bits)   │
└──────────────┴──────────────┴──────────────┘
```

### Page Structure

Pages are the basic unit of allocation:

```
┌─────────────────────────────────────┐
│            Page Header              │
├─────────────────────────────────────┤
│                                     │
│         Object Storage              │
│                                     │
├─────────────────────────────────────┤
│         Free List                   │
└─────────────────────────────────────┘
```

## Collection Process

### 1. Mark Phase

- Start from root objects (stack, globals)
- Mark reachable objects
- Use tri-color marking (white, gray, black)

### 2. Relocate Phase

- Move live objects to new pages
- Update all references

### 3. Sweep Phase

- Free dead objects
- Return pages to allocator

## API

### Runtime Functions

```c
void fax_fgc_init();                    // Initialize GC
void* fax_fgc_alloc(size_t size, ...);  // Allocate memory
void fax_fgc_collect();                  // Trigger collection
void fax_fgc_write_barrier(void* obj, void* ref);  // Write barrier
```

## Configuration

### Constants

```zig
const AGE_THRESHOLD: u8 = 3;                    // Promotion threshold
const NURSERY_SIZE: usize = 4 * 1024 * 1024;   // 4MB nursery
const MAX_ALLOCATION_SIZE: usize = 1024 * 1024 * 1024;  // 1GB max
```

## Performance

- **Allocation**: O(1) bump allocation in nursery
- **Collection**: Parallel marking with thread pool
- **Memory**: Predictable pause times with generational collection
