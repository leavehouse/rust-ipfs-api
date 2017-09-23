extern crate ipfs_api;

use ipfs_api::{IpfsApi, Request};

fn main() {
    let mut ipfs = IpfsApi::default();

    let res = ipfs.commands();
    println!("commands result: {:?}", res);
    /*
    let res = ipfs.config_get("Addresses");
    println!("config_get result: {:?}", res);
    //let res = ipfs.config_show();
    //println!("config_show result: {:?}", res);
    let res = ipfs.id();
    println!("id result: {:?}", res);
    let res = ipfs.version();
    println!("version result: {:?}", res);
    */
    let readme_path = "/ipfs/QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG/readme";
    let res = ipfs.cat(readme_path);
    println!("catting the readme: {:?}", res);

}
