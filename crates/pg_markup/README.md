# `pg_markup`

The crate contains procedural macros to build `pg_console` markup object with a JSX-like syntax

The macro cannot be used alone as it generates code that requires supporting types declared in the
`pg_console` crate, so it's re-exported from there and should be used as `pg_console::markup`

## Acknowledgement

This crate was initially forked from [biome](https://github.com/biomejs/biome).