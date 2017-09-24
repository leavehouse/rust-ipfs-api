extern crate ipfs_api;

use ipfs_api::IpfsApi;
use std::path::Path;

fn main() {
    let mut ipfs = IpfsApi::default();
    let info = ipfs.add(Path::new("wireshark_capture/moloch.txt"))
                   .expect("Error adding");
    ipfs.cat(info.Hash).expect("Error catting");
    ipfs.version().expect("Error getting version");
}
