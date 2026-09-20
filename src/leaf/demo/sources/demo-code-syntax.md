# Code Blocks with Syntax Highlighting

Read code — 40+ languages with colors, line numbers, and frames.

## What Syntax Highlighting is

**Syntax Highlighting** renders fenced code blocks with language-aware coloring, decorative frames, and line numbers.

## Rust

```rust
fn main() {
    let name = "leaf";
    println!("Hello from {}!", name);
}
```

## Python

```python
def fibonacci(n):
    a, b = 0, 1
    for _ in range(n):
        a, b = b, a + b
    return a
```

## JavaScript

```javascript
const greet = (name) => {
    return `Hello, ${name}!`;
};
console.log(greet("leaf"));
```

## Bash

```bash
#!/bin/bash
for file in *.md; do
    leaf --watch "$file" &
done
```
