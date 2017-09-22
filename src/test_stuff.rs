extern crate ipfs_api;

use ipfs_api::{IpfsApi, Request};

fn main() {
    let mut ipfs = IpfsApi::default();

    ipfs.version();
    ipfs.id();
    ipfs.commands();
    ipfs.config_show();
    ipfs.config_get("Addresses");
}
