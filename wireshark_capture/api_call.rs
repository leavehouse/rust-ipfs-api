extern crate ipfs_api;

use ipfs_api::IpfsApi;
use std::path::Path;

fn main() {
    let mut ipfs = IpfsApi::default();
    let add_info = ipfs.add(&[Path::new("wireshark_capture/moloch.txt"),
                              Path::new("Cargo.toml")])
                   .expect("Error adding");

    for info in add_info {
        ipfs.cat(&info.Hash).expect("Error catting");
    }

    ipfs.version().expect("Error getting version");
}
