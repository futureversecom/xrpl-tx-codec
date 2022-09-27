# xrpl-tx-codec

xrpl-tx-codec for binary serializing XRPL transactions (based on xrpl-rust).

It is intended to only support a subset of transaction types for use by the root network xrpl-bridge & ethy-gadget.

Intended for use with `#![no_std]`

## integration tests

The integration tests use xrpl.js as reference implementation.  
ensure you have node & yarn installed.  

```
# install xrpl.js
> yarn
> cargo test --test transaction_decoding
```
