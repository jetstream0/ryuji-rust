Basically translated Ryuji (originally written in Typescript) to Rust, to allow usage in Rust, and also hopefully for increased speed. We'll see...

Translating wasn't very easy, since Rust is a lot stricter. It was pretty fun though.

See an example in `example` directory, or in the tests in `src/lib.rs`. Another example of Ryuji syntax (which uses the Typescript library not the Rust library) can be found at [hedgeblog](http://github.com/jetstream0/hedgeblog).

The library documentation is probably on [docs.rs](https://docs.rs/ryuji_rust/latest/ryuji_rust/), and the templating language docs is [here](https://www.prussiafan.club/posts/ryuji-docs/).
