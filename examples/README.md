# Fax Examples

This directory contains example projects demonstrating various Fax features.

## Available Examples

### calculator/
A simple calculator with basic arithmetic operations, power, and factorial.

```bash
cd calculator
python3 ../faxt/main.py run main.fax
```

**Features demonstrated:**
- Functions with parameters and return values
- Match expressions
- Error handling (division by zero)
- Recursion

### data-structures/
Common data structures and algorithms including arrays, sorting, and searching.

```bash
cd data-structures
python3 ../faxt/main.py run main.fax
```

**Features demonstrated:**
- Arrays and array operations
- Structs
- Algorithms (bubble sort, binary search)
- Iteration patterns

### todo/
A todo list application showing CRUD operations.

```bash
cd todo
python3 ../faxt/main.py run main.fax
```

**Features demonstrated:**
- Structs with multiple fields
- Array manipulation
- State management
- Pattern matching

## Running Examples

All examples can be run with:

```bash
python3 faxt/main.py run examples/<example-name>/main.fax
```

## Creating Your Own

To create a new example:

1. Create a directory: `mkdir examples/my-example`
2. Create `main.fax` with your code
3. Add a README.md explaining the example
4. Update this file to include your example

## Contributing

Have a great example? Submit a PR!
