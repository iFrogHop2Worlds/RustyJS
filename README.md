# RustyJS: JavaScript DSL & Runtime in Rust

A Rust-based JavaScript interpreter and compiler infrastructure designed to execute JavaScript code within Rust environments and compile JS directly to Rust binaries.

## Overview

RustyJS consists of two complementary components:

1. **JS Macro (`js_macro`)** - A procedural macro that parses JavaScript-like syntax and generates Rust code
2. **JS Runtime (`js_runtime`)** - A complete JavaScript runtime implementation in pure Rust

### The Vision

The ultimate goal of RustyJS is to **copy-paste any JavaScript code into a Rust macro and have it just work**, eventually enabling:
- Direct execution of JS code within Rust applications
- Compilation of JavaScript to standalone Rust binaries
- Zero-dependency JavaScript execution in performance-critical systems

---

## Project Structure

```
js_rust_dsl/
├── js_macro/          # Procedural macro for parsing JS syntax
├── js_runtime/        # Complete JS runtime implementation
└── js_example/        # Example usage and tests
```

### Components

#### `js_macro` - JavaScript Parser & Code Generator
- **Purpose**: Transforms JavaScript syntax into Rust bytecode instructions
- **Features**:
    - Full expression parsing (binary ops, unary ops, logical ops, method calls)
    - Statement parsing (declarations, control flow, exception handling)
    - Function declarations and calls
    - Object and array literals
    - Member and index access with assignment support
    - Compound assignment operators (`+=`, `-=`, `*=`, `/=`, `%=`)
    - Increment/decrement operators (`++`, `--`)

**Usage**:
```rust
use js_macro::js;

fn main() {
    js! {
        let x = 5;
        let y = 10;
        console.log(x + y);  // Output: 15
    }
}
```

#### `js_runtime` - JavaScript Execution Engine
- **Purpose**: Executes compiled JavaScript bytecode with full JS semantics
- **Capabilities**:
    - Variable binding with proper scoping (let/const semantics)
    - Dynamic typing with JavaScript type coercion rules
    - Function declarations and invocations
    - Object and array manipulation
    - Method calls with proper `this` context
    - Exception handling (try-catch-finally with proper flow control)
    - Built-in functions and methods:
        - **Global constructors**: `String()`, `Number()`, `Boolean()`, `Error()`, `TypeError()`
        - **Console**: `console.log()`
        - **Math object**: `max()`, `min()`, `floor()`, `ceil()`, `round()`, `abs()`, `pow()`
        - **Array methods**: `push()`, `pop()`, `map()`, `filter()`, `reduce()`, `forEach()`, `includes()`, `join()`
        - **Object methods**: `Object.keys()`, `Object.values()`, `Object.assign()`
        - **JSON**: `JSON.stringify()`, `JSON.parse()`

---

## Current Capabilities

### Supported JavaScript Features

✅ **Data Types**
- Numbers, Strings, Booleans
- Arrays, Objects
- Functions (named & anonymous)
- Null, Undefined
- Error objects with stack traces

✅ **Operators**
- Arithmetic: `+`, `-`, `*`, `/`, `%`
- Comparison: `==`, `!=`, `===`, `!==`, `<`, `>`, `<=`, `>=`
- Logical: `&&`, `||`, `!`
- Assignment & Compound: `=`, `+=`, `-=`, `*=`, `/=`, `%=`
- Postfix: `++`, `--`

✅ **Control Flow**
- `if`/`else if`/`else` statements
- `for` loops with full initialization, condition, increment
- `while` and `do-while` loops
- `return` statements
- `try-catch-finally` with optional catch binding
- `throw` statements

✅ **Object-Oriented Features**
- Object literals with shorthand properties
- Member access (dot notation & bracket notation)
- Method calls with `this` context binding
- Function objects as first-class values

✅ **Advanced Features**
- Callback functions and higher-order functions
- Array methods (`map`, `filter`, `reduce`)
- String concatenation with type coercion
- Proper JavaScript truthiness/falsiness semantics

---

## Real-World Applications

### 1. **Embedded Scripting for Rust Applications**
Execute user-provided JavaScript logic without spawning a separate JavaScript engine:
```rust
// User configuration scripts in JS
js! {
    const config = {
        maxRetries: 3,
        timeout: 5000,
        logLevel: "debug"
    };
}
```

### 2. **Cross-Platform Game Development**
Write game logic in familiar JavaScript, compile to high-performance Rust binaries:
```rust
js! {
    const player = { x: 100, y: 100, health: 100 };
    
    function update(deltaTime) {
        player.x += deltaTime * player.velocity;
        if (player.health <= 0) {
            gameOver();
        }
    }
}
```

### 3. **Data Processing Pipelines**
Chain transformations using JavaScript array methods compiled to Rust:
```rust
js! {
    const results = data
        .filter(item => item.valid)
        .map(item => ({ ...item, processed: true }))
        .reduce((sum, item) => sum + item.value, 0);
}
```

### 4. **Edge Computing & Serverless Functions**
Deploy lightweight JS functions as Rust binaries without runtime overhead:
- Compile JS → Rust bytecode → Binary
- No JavaScript engine dependency
- Static typing guarantees from Rust
- Native performance

### 5. **Browser Automation & Testing**
Execute JavaScript test scripts in a controlled Rust environment:
```rust
js! {
    function testUserFlow() {
        let user = authenticate("test@example.com");
        user.updateProfile({ name: "Updated" });
        return user.verified;
    }
}
```

### 6. **WebAssembly Binding**
Compile JavaScript to WebAssembly via Rust for browser execution with full type safety.

---

## Roadmap

### Phase 1 (Current) ✅
- [x] Core JavaScript parsing and execution
- [x] Basic built-in functions
- [x] Exception handling
- [x] Array and Object support

### Phase 2 (In Progress)
- [ ] Expand built-in library (String methods, Array methods)
- [ ] Arrow functions
- [ ] Async/await support
- [ ] Classes and inheritance
- [ ] Template literals

### Phase 3
- [ ] Direct JS-to-Rust binary compilation
- [ ] WebAssembly code generation
- [ ] Optimization passes for generated code
- [ ] Module system (import/export)

### Phase 4
- [ ] Full ECMAScript compatibility
- [ ] Debugger support
- [ ] Performance profiling tools
- [ ] Package manager integration

---

## Example Usage

See `js_example/src/main.rs` for comprehensive examples covering:
- Functions and closures
- Control flow (if/else, loops)
- Arrays with methods (map, filter, reduce)
- Objects with methods
- Exception handling (try-catch-finally)
- Built-in functions (Math, JSON, Object, String, Number)

---

## Architecture

**Compilation Pipeline**:
```
JavaScript Code → Parser → IR → Bytecode → Runtime Execution
```

1. **Parser** (`js_macro/src/lib.rs`): Converts JS syntax to an AST
2. **Intermediate Representation** (`js_macro/src/ir.rs`): Lowers AST to IR
3. **Bytecode Compiler** (`js_runtime/src/lib.rs`): Compiles IR to stack-based bytecode
4. **Execution Engine** (`js_runtime/src/lib.rs`): Interprets bytecode with proper scoping and control flow

---

## Why RustyJS?

| Aspect | Benefit |
|--------|---------|
| **Performance** | No external JS engine required; compile to native Rust binaries |
| **Portability** | Ship JavaScript logic as part of Rust binaries—no runtime dependency |
| **Type Safety** | Rust's type system ensures memory safety; compiled output is verifiable |
| **Integration** | Execute JS natively from Rust; call Rust from JavaScript (future) |
| **Deployment** | Single binary distribution with embedded JavaScript execution |

---

## Contributing

We're actively building toward full JavaScript compatibility. Contributions welcome for:
- Additional built-in functions
- Performance optimizations
- Syntax feature expansion
- Test coverage


