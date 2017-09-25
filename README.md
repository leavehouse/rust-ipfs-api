# rust-ipfs-api

WIP. A Rust interface to the [ipfs HTTP API](https://ipfs.io/docs/api/).

# Usage example

```rust
extern crate ipfs_api;

use ipfs_api::IpfsApi;
use std::io::{self, Write};

fn main() {
    let mut ipfs = IpfsApi::default();

    let readme_path = "/ipfs/QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG/readme";
    match ipfs.cat(readme_path) {
        Err(e) => panic!("Error catting readme: {:?}", e),
        Ok(res) => {
            println!("catting the readme");
            io::stdout().write(&res[..]).expect("Error writing to stdout");
        }
    }

    match ipfs.version() {
        Err(e) => panic!("Error getting version: {:?}", e),
        Ok(info) => println!("ipfs version: {}, repo version: {}, system: {}",
                             info.Version, info.Repo, info.System),
    }
}
```
