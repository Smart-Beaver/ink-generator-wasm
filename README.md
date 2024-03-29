<div align="center">

![logoVMov-sm](https://github.com/Smart-Beaver/.github/assets/8248700/d9d963fd-e6f7-46cb-9fce-07edc6a6d33f)

  <strong>A WASM module for generating smart contract code in <a href="https://use.ink/">ink!</a></strong>

  <p>
    <a href="https://travis-ci.org/rustwasm/wasm-pack-template"><img src="https://img.shields.io/travis/rustwasm/wasm-pack-template.svg?style=flat-square" alt="Build Status" /></a>
  </p>

  <h3>
    <a href="https://rustwasm.github.io/docs/wasm-pack/tutorials/npm-browser-packages/index.html">Read docs</a>
  </h3>

  Built with 🦀 by:
  <ul style="text-align: center; list-style: none; padding-left: 0px">
  <li><a href="https://maciekmalik.pl/">Maciek Malik</a></li>
  <li><a href="https://codewithiza.com/">Izabela Łaszczuk</a></li>
  <li><a href="https://www.blockydevs.com/">BlockyDevs</a></li>
  </ul>
</div>

## About

This repository is the main implementation of the rust code generator. It can produce code ready for deployment based on code fragments located [here]().
This implementation uses [syn](https://crates.io/crates/syn) rust crate.
It is designed to be used as a web-assembly module executed on the client side.  


Be sure to check out [other `wasm-pack` tutorials online][tutorials] for other
templates and usages of `wasm-pack`.

[tutorials]: https://rustwasm.github.io/docs/wasm-pack/tutorials/index.html


### 🛠️ Build with `wasm-pack build`

```
wasm-pack build
```

### 🔬 Test in Headless Browsers with `wasm-pack test`

```
wasm-pack test --headless --firefox
```

### 🎁 Publish to NPM with `wasm-pack publish`

```
wasm-pack publish
```

### 🛠️ Testing merged Smart Contracts code

First generate code of merged contracts:

```
cargo run init
```

It will create code of the SC with tested extension and save it to 
`contracts/[STANDARD]/extension/tests/[EXTENSION]/src` directory.

This command uses the same method as our front-end tool, so the code being tested is the same as the one that you can see.

Then run this command:
```
cargo test --manifest-path contracts/Cargo.toml --features "contract","test-only"
```

## 🚴 Usage

For details about integrating compiled wasm module into your front-end app see this [docs](https://rustwasm.github.io/book/game-of-life/hello-world.html)

## 🔋 Batteries Included

* [`wasm-bindgen`](https://github.com/rustwasm/wasm-bindgen) for communicating
  between WebAssembly and JavaScript.
* [`console_error_panic_hook`](https://github.com/rustwasm/console_error_panic_hook)
  for logging panic messages to the developer console.
* `LICENSE-APACHE` and `LICENSE-MIT`: most Rust projects are licensed this way, so these are included for you

## License

Licensed under:
* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)


### Contribution

Please check [Contributing docs](https://smart-beaver.github.io/#contributing) for details.