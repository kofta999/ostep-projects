# OSTEP Projects (Rust)

This repository contains my implementations of selected projects from  
**Operating Systems: Three Easy Pieces (OSTEP)**.

I worked through most of the OSTEP lectures and book material first, and then implemented the projects to reinforce the concepts through hands-on systems programming. The focus is on correctness, clarity, and direct interaction with Unix primitives rather than feature completeness or polish.

All projects are implemented in **Rust**, using low-level OS interfaces where appropriate.

---

## Projects

### Unix-like Shell (`wish`)
A minimal shell implementation supporting:

- Built-in commands: `cd`, `exit`, `path`
- Execution of external programs via `fork` / `exec`
- Output redirection (`>`)
- Parallel command execution (`&`)
- Explicit path management (no implicit `$PATH` lookup)

The shell is intentionally simple and closely follows the OSTEP specification, with an emphasis on understanding process creation, waiting semantics, and file descriptor manipulation.

---

### `stat` (simplified)
A simplified reimplementation of the Unix `stat` utility, focusing on:

- File metadata retrieval
- Permissions, ownership, and size
- Interaction with filesystem-related system calls

---

### `tail` (simplified)
A basic version of `tail` that demonstrates:

- File I/O at the OS level
- Efficient reading of file contents
- Handling edge cases around file size and line boundaries

---

## Goals

- Reinforce core OS concepts through implementation
- Gain practical experience with Unix process and filesystem APIs
- Write clear, explicit systems code in Rust
- Build a growing collection of small, focused OS projects

This repository is expected to evolve as I continue working through additional OSTEP projects and more advanced exercises.

---

## Notes

- These implementations prioritize learning and correctness over completeness.
- Error handling follows the OSTEP project guidelines (single global error message where required).
- The code intentionally avoids higher-level abstractions when lower-level primitives are the learning goal.

---

## References

- *Operating Systems: Three Easy Pieces*  
  https://pages.cs.wisc.edu/~remzi/OSTEP/

---

