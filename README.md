# xrpl-tiny-codec

xrpl-tiny-codec for binary serializing XRPL transactions (based on xrpl-rust).

It is intended to only support a subset of transaction types for use by the root network ethy-gadget.
`no_std` is not required necessary as the ethy-gadget resides on the client.

## integration tests

The integration tests use xrpl.js as reference implementation.  
ensure you have node & yarn installed.  

```
# install xrpl.js
> yarn
> cargo test --test transaction_decoding
```