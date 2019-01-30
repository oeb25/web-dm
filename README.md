# Web DM

A set of tools for visualizing and solve discrete mathematic problems

## Origin and purpose

At uni I had a discrete mathematics [course](http://kurser.dtu.dk/course/01017), where the first part of the couse was centered around propositional logic followed by first-order logic. For this I developed a tool to visualize, and solve these problems, highly focused around the tableaux method. I implemented a parser, a solver and a visualizer all written in Rust, running in the browser using WASM.

Now the couse is complete, and I therefor decided to publish my tool for anybody to use.

This tool was manly used to serve me, and solve the problems I encouterd, in the way I saw fit. This is highly reflected in the UI and the code, both of which are very adhoc. You are very welcome to use/modify/destribute this tool as you see fit.

## Logic syntax

+-------+--------+
| Logic | Web DM |
+-------+--------+
|   ¬x  |   !x   |
| a ∨ b |  a | b |
| a ∧ b |  a & b |
| a → b |  a > b |
| a ↔ b |  a = b |
| ∃x(x) |  .x(x) |
| ∀x(x) |  \x(x) |
+-------+--------+

## Building and running

You need to have Rust installed, and `cargo-web` installed (see [yew](https://github.com/DenisKolodin/yew#development-setup) for details).

Then `cd` to `web/` and run

```
cargo web start --target=wasm32-unknown-unknown
```

When this is complete, you can now access the app at [`[::1]:8000`](http://[::1]:8000/).

## Support

I don't plan to further develop this for my own purpos, since it has done its job for me, but if you find issues or flaws, you are welcome to open an issue, and we can discuss it further!
