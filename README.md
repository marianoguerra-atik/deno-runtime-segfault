# deno-runtime-segfault

reproduction of issue

## Reproduce

```sh
cargo run
```

## Avoid it

Comment worker field in `struct Handler` and `Handler { ... }` constructor and run it again.
