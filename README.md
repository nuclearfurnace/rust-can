# can

A general purpose library of common CAN types.

[![Code of Conduct][conduct-badge]][conduct]
[![MIT licensed][license-badge]](#license)
[![Documentation][docs-badge]][docs]
![last-commit-badge][]
![contributors-badge][]

[conduct-badge]: https://img.shields.io/badge/%E2%9D%A4-code%20of%20conduct-blue.svg
[conduct]: https://github.com/nuclearfurnace/rust-can/blob/master/CODE_OF_CONDUCT.md
[license-badge]: https://img.shields.io/badge/license-MIT-blue
[docs-badge]: https://docs.rs/can/badge.svg
[docs]: https://docs.rs/can
[last-commit-badge]: https://img.shields.io/github/last-commit/nuclearfurnace/rust-can
[contributors-badge]: https://img.shields.io/github/contributors/nuclearfurnace/rust-can


## code of conduct

**NOTE**: All conversations and contributions to this project shall adhere to the [Code of Conduct][conduct].

# what's it all about?

`can` provides foundational types for working with CAN networks, such as identifiers, frames,
and filters.  These types are simple and straightforward on their own, but this crate exists to
provides a common dependency that all other crates can hopefully depend on.  In turn, it could
unlock seamless interoperation between those crates without requiring boilerplate or constant
reimplementation of conversion methods and excess dependencies.

# contributing

We're always looking for users who have thoughts on how to make `can` better, or users with
interesting use cases.  Of course, we're also happy to accept code contributions for outstanding
feature requests! 😀
