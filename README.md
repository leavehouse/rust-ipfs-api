# rust-ipfs-api

A Rust interface to the [ipfs HTTP API](https://ipfs.io/docs/api/).

# Usage
This is very much a WIP, but currently can be used like so:

```rust
extern crate ipfs_api;

use ipfs_api::IpfsApi;
use std::io::{self, Write};

fn main() {
    let mut ipfs = IpfsApi::default();
    let readme_path = "/ipfs/QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG/readme";
    let res = match ipfs.cat(readme_path) {
        Err(e) => panic!("Could not cat readme: {:?}", e),
        Ok(res) => {
            println!("catting the readme");
            io::stdout().write(&res[..])
        }
    };
}
```
