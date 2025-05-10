## Rust playground
Welcome to **TypeRust**! This is a simple Rust playground where you can build or run your Rust code and share it with others.

There are a few things to keep in mind before using it:
* Code execution time is limited and if it takes too long to complete it will be interrupted.
* Your program cannot use too much memory. If it exceeds the limit it will be interrupted.
* Since the program runs in a sandbox, it doesn't have access to filesystem and/or network. Of course simply building code is fine.

## Environment

* The code is compiled to `wasm32-wasi` target and is run in a [`wasmtime`](https://github.com/bytecodealliance/wasmtime) instance.
* The latest stable version of Rust with 2021 edition is used.
* There is no way to install crates (yet).

## Development

### Tech
TypeRust playground is powered by [`Svelte`](https://svelte.dev/) and [CodeMirror](https://codemirror.net/6/) editor on frontend and [`axum`](https://github.com/tokio-rs/axum) (and its ecosystem) on backend. [`wasmtime`](https://github.com/bytecodealliance/wasmtime) is used to create ephemeral WASM virtual machines to run user code. The whole thing is deployed to [Fly.io](https://fly.io/).

### Source code
You can find source code on Github: [https://github.com/jlkiri/typerust](https://github.com/jlkiri/typerust).

## About the author
This playground was created by [Kirill Vasiltsov](https://www.kirillvasiltsov.com/).

### Sponsorship
Currently I (the author) pay for the infrastructure out of my own pocket. It is not much but any help is appreciated. Sponsoring via Github is not available at the moment, but you can use my [PayPal profile](https://paypal.me/jlkiri) if you want to help. Anyone with one-time payment of 10$ or more gets:

* A huge Thank You from me
* Optional advice on working as a software engineer in Japan
* Optional advice on contributing to OSS projects

To receive advice contact me at email address on my [personal homepage](https://www.kirillvasiltsov.com/).
