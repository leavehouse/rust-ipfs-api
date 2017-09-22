extern crate ipfs_api;

use ipfs_api::{IpfsApi, Request};

fn main() {
    let mut ipfs = IpfsApi::default();

    let req = Request::new(&ipfs.config, String::from("version"), vec![]);
    ipfs.send_request(&req);
}
