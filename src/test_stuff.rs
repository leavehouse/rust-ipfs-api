extern crate ipfs_api;

use ipfs_api::{IpfsApi, unmarshal, AddInfo, CommandInfo, IdInfo, VersionInfo};
use std::io::{self, Write};

fn main() {
    let mut ipfs = IpfsApi::default();
    if let Err(e) = run_commands(&mut ipfs) {
        panic!("Error making API request: {:?}", e)
    }
}

fn run_commands(ipfs: &mut IpfsApi) -> Result<(), ipfs_api::RequestError> {
    let chunk = ipfs.commands()?;
    let command_info: CommandInfo = unmarshal(&chunk).expect("could not unmarshal");
    println!("commands result: {:?}", command_info);

    println!("~~~~~~~~~~~~~~~~~~~~~");

    /* TODO: update these
    let res = ipfs.config_get("Addresses");
    println!("config_get result: {:?}", res);

    let res = ipfs.config_show();
    println!("config_show result: {:?}", res);

    */

    let chunk = ipfs.id()?;
    let id_info: IdInfo = unmarshal(&chunk).expect("could not unmarshal");
    println!("id result: {:?}", id_info);

    println!("~~~~~~~~~~~~~~~~~~~~~");

    let readme_path = "/ipfs/QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG/readme";
    let out_bytes = ipfs.cat(readme_path)?;
    io::stdout().write(&out_bytes[..]).unwrap();

    println!("~~~~~~~~~~~~~~~~~~~~~");

    let path = std::path::Path::new("lorem_ipsum.txt");
    let chunk = ipfs.add(&[path])?;
    let add_info: AddInfo = unmarshal(&chunk[0]).expect("could not unmarshal");
    println!("added file {:?}", path);
    println!("add info: {:?}", add_info);

    println!("~~~~~~~~~~~~~~~~~~~~~");

    let chunk = ipfs.version()?;
    let version_info: VersionInfo = unmarshal(&chunk).expect("could not unmarshal");
    println!("version result: {:?}", version_info);
    Ok(())
}
