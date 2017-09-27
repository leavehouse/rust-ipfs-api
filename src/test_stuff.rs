extern crate ipfs_api;
extern crate tokio_core;

use ipfs_api::{IpfsApi, unmarshal, AddInfo, CommandInfo, IdInfo, ObjectInfo,
               ObjectLinkInfo, VersionInfo};
use std::io::{self, Write};
use tokio_core::reactor;

fn main() {
    let core = reactor::Core::new().unwrap();
    let mut ipfs = IpfsApi::default(&core.handle());
    if let Err(e) = run_commands(core, &mut ipfs) {
        panic!("Error making API request: {:?}", e)
    }
}

fn run_commands(mut core: reactor::Core, ipfs: &mut IpfsApi) -> Result<(), ipfs_api::RequestError> {
    /*
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
    */

    let obj_get = ipfs.object_get("QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG");
    let chunk = core.run(obj_get)?;
    let obj_info: ObjectInfo = unmarshal(&chunk).expect("could not unmarshal");
    println!("object get result:");
    for obj_link in obj_info.Links.iter() {
        println!(" Link {{ name: {}, hash: {}, size: {} }}", obj_link.Name,
                 obj_link.Hash, obj_link.Size);
    }

    /*
    let readme_hash = obj_info.Links[4].Hash;
    */

    Ok(())
}
