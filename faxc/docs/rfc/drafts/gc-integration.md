# [RFC] GC Integration Design

## Summary

Design the Fax Garbage Collector (FGC) for low-latency concurrent collection.

## Motivation

Systems languages need memory safety without manual management overhead.

## Detailed Design

### Generational Collection
- Nursery: Bump allocation, frequent minor GC
- Tenured: Mark-sweep, infrequent major GC

### Write Barriers
- Card marking for cross-generation references
- Incremental marking during mutator execution

### Safepoints
- Cooperative thread suspension
- Stack maps for root identification

## Implementation Plan

1. Basic allocator (bump pointer)
2. Mark-sweep collector
3. Generational extensions
4. Concurrent collection
5. Optimizations (compaction, etc.)

## Status

Draft - Awaiting review