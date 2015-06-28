# Burrito: a monadic IO interface.

Burrito is a monad. Monads are [basically burritoes](). This monad wraps an IO
handle, hiding its potential state of failure from you for as long as you want.
This isn't _exactly_ like the IO monad from Haskell, but its very similar
conceptually.

[Documentation!]()

### Creating a Burrito

You can instantiate a burrito one of several ways:

* The `burrito()` function will return a Burrito wrapping the stdio handles.
* `Burrito::wrap()` takes an `io::Result<T>` to create a burrito wrapping that
`T`. `io::Result<T>` is the return type of most functions which create IO
handles.
* `Burrito::wrap_func()` takes a function which returns an `io::Result<T>` and
wraps the IO handle returned by that function.
* Several standard IO types implement thet traits `FromPath` or `FromAddr`.
`Burrito::from_path` and `Burrito::from_addr` will transform a path or socket
address into an IO handle and wrap a burrito around it.

Note that `File`'s implementation of `FromPath` will open a file with read,
write, and create all set to `true`. If you wish to open a file with different
options, you will want to use `Burrito::wrap()`.

### Using a Burrito

If the handle inside the burrito implements `Read`, `Write`, `Seek`, or
`BufRead`, the burrito will implement similar methods to those defined on that
trait, though the signatures will differ. The return value of those functions
will then be stored inside the burrito, accessible through the `and_then()`
method which burrito implements. As a simple example, this code will echo once
on stdin/stdout.

```rust
burrito().read_line().and_then(|echo, burrito| burrito.print_line(&echo));
```

More information is available in the API docs.

### Non-blocking IO

Burrito currently is built on top of the standard library's io module, which
is intended for blocking IO. Extensions may be forthcoming which will implement
non-blocking burritoes, probably on top of `mio`.

### Licensing.

This library is licensed under the GPL version 3 or greater with the CLASSPATH
linking exception.
