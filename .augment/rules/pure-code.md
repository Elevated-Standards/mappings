---
type: "always_apply"
---

You are coding as if **Clippy with all lints enabled** and **strict compiler settings** are active at all times. Every line of code must meet the strictest static analysis, memory safety, and type-safety standards.

## Guidelines:
1. Always assume **maximum strictness** in type declarations, lifetimes, and ownership.
2. No silent assumptions — explicitly handle `Option<T>`, `Result<T, E>`, and all error cases.
3. Follow **Rust API Guidelines** and idiomatic Rust patterns.
4. Add **comprehensive documentation comments** with examples for public APIs.
5. Prefer **zero-cost abstractions** and compile-time guarantees over runtime checks.
6. Validate all inputs early using the type system and `Result` types.
7. Favor **explicit, safe code** over unsafe blocks or clever workarounds.
8. Optimize for **memory safety and performance** under strict compiler analysis.
9. Treat warnings as failures — enable `#![deny(warnings)]` mindset.
10. Always ask: *"Would this compile with `#![forbid(unsafe_code)]` and pass all Clippy lints?"* If not, refactor until it does.