# Documentation Structure

This document defines the mandatory structure for all documentation generated
in the `/docs` directory. Every documentation agent **must** follow this
structure when creating or updating files.

## Directory Layout

```
docs/
├── getting-started.md        # Quick-start guide for new users
├── configuration.md          # GA configuration options and builder API
├── chromosomes.md            # Chromosome types (Binary, Range)
├── genotypes.md              # Genotype definitions and usage
├── operators/
│   ├── selection.md          # Selection operators (tournament, roulette, random)
│   ├── crossover.md          # Crossover operators (uniform, multipoint, cycle)
│   ├── mutation.md           # Mutation operators (swap, scramble, inversion, value)
│   └── survivor.md           # Survivor selection (age-based, fitness-based)
├── fitness.md                # Fitness evaluation and custom fitness functions
├── population.md             # Population management
├── traits.md                 # Core traits (Gene, Chromosome, GeneticAlgorithm)
├── validators.md             # Validation system
├── examples.md               # End-to-end examples (knapsack, n-queens, etc.)
└── api-reference.md          # Full API reference summary
```

## Document Template

Every documentation file **must** follow this template:

```markdown
# {Title}

> Brief one-line description of the topic.

## Overview

A 2-4 paragraph introduction explaining:
- What this module/feature does
- When and why a user would need it
- How it fits into the overall library architecture

## Key Concepts

Explain the core types, traits, or abstractions relevant to this topic.
Use tables for parameter/field listings:

| Field        | Type     | Description                          |
|-------------|----------|--------------------------------------|
| `field_name` | `Type`   | What this field controls             |

## Usage

### Basic Example

```rust
// A minimal, runnable code example
```

### Advanced Example

```rust
// A more complete example showing real-world usage
```

## API Reference

Document every public item (struct, enum, trait, function) that belongs
to this topic:

### `StructName`

Description of the struct.

**Fields / Methods:**

| Name       | Signature            | Description               |
|-----------|---------------------|---------------------------|
| `new`      | `fn new(...) -> Self`| Creates a new instance    |

## Related

- Links to related documentation files
- Links to relevant source code or examples
```

## Rules for Agents

1. **File naming** -- Use `snake_case.md` for file names. Subdirectories for
   grouped topics (e.g., `operators/`).
2. **Code examples** -- Every document must include at least one Rust code
   example with ```` ```rust ```` fencing. Examples must be realistic and
   consistent with the library's current API.
3. **Audience** -- Write for an external Rust developer who has never seen
   this codebase. Assume familiarity with Rust but not with genetic algorithms.
4. **Completeness** -- All public types, traits, functions, and their parameters
   must be documented when relevant to the topic.
5. **No redundancy** -- Do not duplicate content across files. Reference other
   docs instead (e.g., "See [Traits](traits.md) for details on the `Gene` trait").
6. **Consistency** -- Use the same heading hierarchy and section ordering
   across all files.
7. **Updates** -- When updating a file, preserve existing sections that are
   still accurate. Add or modify only what the code changes require.
8. **Deletions** -- Only delete a documentation file when the underlying module
   or feature has been entirely removed from the codebase.
