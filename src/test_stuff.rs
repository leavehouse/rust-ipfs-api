extern crate ipfs_api;

use ipfs_api::IpfsApi;
use std::io::{self, Write};

fn main() {
    let mut ipfs = IpfsApi::default();
    /*
    let res = ipfs.commands();
    println!("commands result: {:?}", res);
    let res = ipfs.config_get("Addresses");
    println!("config_get result: {:?}", res);

    let res = ipfs.config_show();
    println!("config_show result: {:?}", res);

    let res = ipfs.id();
    println!("id result: {:?}", res);

    let res = ipfs.version();
    println!("version result: {:?}", res);
    */
    let readme_path = "/ipfs/QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG/readme";
    match ipfs.cat(readme_path) {
        Err(e) => panic!("Could not cat readme: {:?}", e),
        Ok(res) => {
            println!("catting the readme");
            io::stdout().write(&res[..]).unwrap();
        }
    }

    println!("~~~~~~~~~~~~~~~~~~~~~");

    let path = std::path::Path::new("lorem_ipsum.txt");
    match ipfs.add(path) {
        Err(e) => panic!("ipfs add failed: {:?}", e),
        Ok(res) => {
            println!("added {:?}", path);
            println!("{:?}", res);
        }
    }
}
