
# Rsgateway - Building an API gateway from scratch

## Goals

To understand the work that's abstracted away by most HTTP frameworks, I intend to build an UNIX only API gateway in Rust from scratch bringing in the miminimal amount of dependencies and thus building as much as possible from scratch (e.g. dns, http, json marshalling and unmarshalling, redis client, ....)

In those implementations, bringing in [tokio.rs](https://tokio.rs/) and dev dependencies ([rstest](https://docs.rs/rstest/latest/rstest/), [criterion](https://bheisler.github.io/criterion.rs/book/)) to facilitate development are allowed. 

Since most RFCs cover large amount of edge cases, implementations based on specs will be best effort; meaning that obscure / deprecated parts of the protocol will be ignored.

