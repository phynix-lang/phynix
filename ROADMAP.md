# ROADMAP

### 1. phynix-lex

- Input: `&str` source
- Output: `Vec<Token { kind, span }>`
- Goals:
    - [X] Zero heap allocations
    - [X] Stateless: Same input -> Same tokens
    - [X] BOM skip
    - [X] File-mode handling (`<?php`, `<?phx`, `<?phxt`)
- Features:
    - [X] Regex-based via `logos`
    - [X] Configurable "mode" flag (`PHP`, `PHX`, `PHXT`)
    - [X] Incremental-friendly spans
- Optimizations:
    - [X] Borrow slices, avoid cloning `lexeme`
    - [ ] Possible interning for identifiers

### 2. phynix-parse

- Input: Tokens
- Output:
    - `AST`
    - `Vec<Diagnostic>`
- Core rules:
    - [X] Pratt parser for expressions, table-driven for statements
    - [X] Graceful error recovery
- Features:
    - [ ] Handles both `.phx` and `.php` syntactic modes
    - [X] Skips trivia, stores spans on every node for mapping to source
    - [X] Produces `ParseResult { ast, diagnostics }`
- Optimizations:
    - [ ] Arena allocator for nodes (tight memory, cache-friendly)
    - [ ] Optional incremental parse hooks later (CTS reuse)
    - [ ] No `String` duplication - spans reference source

### 3. phynix-analyze

- Input: `AST`
- Output:
    - Typed Model / Symbol Table / IR
    - `Vec<Diagnostics>` (Semantic/Type Errors, deprecations)
- Responsibilities:
    - Name resolution & scope stack
    - Type inference / checking
    - Deprecation & legacy detection
    - LSP data model: symbol ranges, completions, hover info
- Optimizations:
    - Pure, deterministic function per file - easy caching
    - Memoization by node hash -> incremental re-analysis
    - Shared interner for types/symbols to dedup memory
- Lexer extensions:
    - Cross-file dependency graph (imports, composer packages)
    - Flow analysis, control-flow graph (for JIT hints)

### 4. phynix-vm

- Input: `AST` or typed IR
- Output: Program result / side effects
- Stage A - Tree-walking interpreter:
    - Directly evaluate AST nodes
    - Scoped `EnvStack<HashMap>` for variables/functions
    - Typed `Value` enum (`Int`, `String`, `Bool`, `Vec`, `Map`, `Func`, `Null`)
- Stage B - Bytecode compiler + VM:
    - Compile typed AST to `Vec<Instr>`
    - Stack-based execution, register caching
    - Separate constant pool for literals/functions
- Stage C - JIT / multi-thread runtime
    - JIT via Cranelift
    - Concurrent green threads / async coroutines
    - Sandboxed extensions for FFI & I/O
- Optimizations:
    - Separate immutable const pool
    - Inline-cache for property access & function lookup
    - Deterministic instruction layout (no boxing)

### 5. phynix-lsp

- Input: Source text
- Uses: `lex`, `parse`, `analyze`
- Output: Diagnostics, completions, hovers, go-to-definition
- Features:
    - Full live diagnostics
    - Incremental parsing/analyzing via document snapshots
    - Code actions
    - Hover shows type + docstring
- Optimizations:
    - Cache `ParseResult` + `AnalysisResult` per buffer
    - Diff-based reparse: only changed regions
    - Thread-pooled tasks for responsiveness

### 6. phynix-cli

- Sub-commands:
    - `lex` (Dev)
    - `parse` (Dev)
    - `check`
    - `run`
    - `compile` (Dev)
    - `lsp`
    - `fix`
    - `lint`
    - Composer integration

### 7. phynix-core

- Contains:
    - `Diagnostic`, `Severity`, `Span`, `Position`, `SymbolId`
    - Interners, common types (`TypeRef`, `Value`, etc.)
    - Config flags (strict / compat mode)
    - Logging, profiling, benchmarks
