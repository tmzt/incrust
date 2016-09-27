# incrust
Zero-cost incremental-rendering isometric web templates in Rust

Inspired by [Khufu](http://tailhook.github.io/khufu/) and [IncrementalDOM](http://google.github.io/incremental-dom/#about).


###What it is
*incrust* offers isometric rendering of reactive web applications, without server-side javascript like node.

#### *Note: this is a work in progress, you will see some demo functionality at this point but the template language is not fully implemented and only the basic level of code generation works. There is currently no standalone codegen program, so you will need to use a nightly of Rust.*

###Why
Because you shouldn't need an entire language runtime to render a page when you already have a web framework in a fast language like Rust. You also shouldn't have to live in the past and choose either server-side or client-side rendering. Why not do both from the same code?

###How it works
Much like Khufu, there is a template language combining HTML and another simplified language. For us it's a small subset of Rust. Integration with common libraries like Redux can be accomplished, with data transfer requests from the Rust backend served with the same business logic as the initial response. You write your template code in a basic Rust-like DSL, and write your backend storage and database-integration any way you like. You can access your Redux store directly from the DSL.

See Khufu for an idea of what we are doing.

###Is it fast?
It will be. The server-side rendering code will all be compiled Rust, the client-side javascript will be generated from the same sources and cached as a compiled JS file. (This isn't using asm.js or wasm, just normal JS)

###What else does Rust get me?
For one, strongly typed objects in your JS code. Though this isn't enforced on the client side, the code generation will only allow access to objects backed by a *struct*. This can be disabled for an object by using the special *Any* type.

It will also be possible to generate TypeScript interface files and <em>*.d.ts</em> files.

###Run the demo

1. Checkout this repo into a directory
2. *cd* into that directory
3. Execute *npm install* to get incremental-dom.js
4. Execute *cargo run --example demo*

Then visit http://127.0.0.1:6767/ or the URL shown at the command prompt!
