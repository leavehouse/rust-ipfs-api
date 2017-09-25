extern crate futures;
extern crate hyper;
extern crate multiaddr;
extern crate multipart_legacy_client;
extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate serde_json;
extern crate tokio_core;


use futures::{Future, Stream};
use multiaddr::{Multiaddr, Protocol};
use multipart_legacy_client::send_new_post_request;
use tokio_core::reactor;
use std::str;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

fn is_ip(p:Protocol) -> bool {
    match p {
        Protocol::IP4 | Protocol::IP6 => true,
        _ => false,
    }
}

// Comment from go-multiaddr-net:
//
// "IsThinWaist returns whether a Multiaddr starts with "Thin Waist" Protocols.
// This means: /{IP4, IP6}[/{TCP, UDP}]"
fn is_thin_waist(m: &Multiaddr) -> bool {
    let protocol = m.protocol();
    if protocol.len() == 0 {
        return false
    }
    let p1 = protocol[0];
    if !is_ip(p1) {
        return false
    }
    if protocol.len() == 1 {
        return true
    }
    let p2 = protocol[1];
    is_ip(p1) && (p2 == Protocol::TCP || p2 == Protocol::UDP || is_ip(p2))
}

// TODO: args could be an Iterator?
// TODO: command could be some U where U: AsRef<str> to avoid allocation?
pub struct Request<T> {
    api_base: String,
    command: String,
    args: Vec<String>,
    other_data: T,
}

impl Request<()> {
    pub fn new(cfg: &Config, command: String, args: Vec<String>) -> Request<()> {
        Request::new_with_data(cfg, command, args, ())
    }
}

impl<T> Request<T> {
    pub fn new_with_data(cfg: &Config, command: String, args: Vec<String>,
                         other_data: T)
            -> Request<T> {
        let api_base = format!("http://{}:{}{}", cfg.host, cfg.port,
                               cfg.api_path);
        Request { api_base, command, args, other_data }
    }

    pub fn make_uri_string(&self) -> String {
        let args_str = self.args.iter()
                                .map(|ref arg| format!("arg={}", arg))
                                .collect::<Vec<_>>()
                                .join("&");
        format!("{}/{}?{}", self.api_base, self.command, args_str)
    }

    // TODO: options
    pub fn get_uri(&self) -> hyper::Uri {
        let uri_string = self.make_uri_string();
        match uri_string.parse() {
            Err(err) => panic!("Parse of {} failed: {}", uri_string, err),
            Ok(uri) => uri,
        }
    }

    pub fn new_hyper_request(&self, method: hyper::Method) -> hyper::Request {
        hyper::Request::new(method, self.get_uri())
    }

}

pub struct Config {
    // no trailing slash. should enforce in constructor?
    api_path: String,
    host: String,
    port: u16
}

impl Config {
    fn default() -> Config {
        Config {
            api_path: String::from("/api/v0"),
            host: String::from("localhost"),
            port: 5001
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct AddInfo {
    pub Name: String,
    pub Hash: String,
    pub Size: String,
}

#[derive(Debug, Deserialize)]
pub struct VersionInfo {
    pub Version: String,
    Commit: String,
    pub Repo: String,
    pub System: String,
    Golang: String,
}

#[derive(Debug, Deserialize)]
pub struct IdInfo {
    ID: String,
    PublicKey: String,
    Addresses: Vec<String>,
    AgentVersion: String,
    ProtocolVersion: String,
}

#[derive(Debug, Deserialize)]
pub struct CommandInfo {
    Name: String,
    Subcommands: Vec<CommandInfo>,
    Options: Vec<CommandNames>,
}

#[derive(Debug, Deserialize)]
pub struct CommandNames {
    Names: Vec<String>
}

pub type RequestResult<T> = Result<T, RequestError>;


pub struct IpfsApi {
    config: Config,
    core: reactor::Core,
    client: hyper::Client<hyper::client::HttpConnector>,
}

impl IpfsApi {
    pub fn default() -> IpfsApi {
        Self::new(Config::default())
    }

    pub fn new(cfg: Config) -> IpfsApi {
        let core = reactor::Core::new().unwrap();
        let client = hyper::Client::new(&core.handle());
        IpfsApi {
            config: cfg,
            core: core,
            client: client,
        }
    }

    pub fn new_request(&self, command: String, args: Vec<String>) -> Request<()> {
        Request::new(&self.config, command, args)
    }

    pub fn new_multipart_request<'a>(&self, command: String, args: Vec<String>,
                                     files: Vec<&'a Path>)
            -> Request<Vec<&'a Path>> {
        Request::new_with_data(&self.config, command, args, files)
    }

    pub fn send_request(&mut self, request: &Request<()>)
                        -> RequestResult<hyper::Chunk> {
        let hyper_req: hyper::Request = request.new_hyper_request(hyper::Method::Post);
        //req.headers_mut().set(ContentType::json());
        //req.headers_mut().set(ContentLength(json.len() as u64));
        //hyper_req.set_body(request);
        let post = self.client.request(hyper_req).and_then(|res| {
            res.body().concat2()
        });

        Ok(self.core.run(post)?)
    }

    // multipart-async doesnt seem to be ready, so this is synchronous for now
    pub fn send_request_multipart(&mut self, request: &Request<Vec<&Path>>)
                                  -> RequestResult<Vec<u8>> {
        let url = request.make_uri_string();
        Ok(send_new_post_request(url, &request.other_data[..])?)
    }

    fn request_string_result<T, U>(&mut self, command: T, args: Vec<U>)
                                   -> RequestResult<String>
                                   where T: ToString,
                                         U: ToString {
        self.request(command, args)
            .map(|chunk| str::from_utf8(&chunk).unwrap().to_string())
    }


    pub fn request<T,U>(&mut self, command: T, args: Vec<U>)
            -> RequestResult<hyper::Chunk> where T: ToString, U: ToString {
        let req = self.new_request(command.to_string(),
                                   args.into_iter()
                                       .map(|a| a.to_string())
                                       .collect());
        self.send_request(&req)
    }

    pub fn request_multipart<T>(&mut self, command: T, files: Vec<&Path>)
            -> RequestResult<Vec<u8>> where T: ToString {
        let req = self.new_multipart_request(command.to_string(),
                                             vec![] as Vec<String>, files);
        self.send_request_multipart(&req)
    }

    /*** start of API calls ***/
    // TODO: these should be on an HttpApi Trait, which would conform with
    // https://github.com/ipfs/interface-ipfs-core#api

    // TODO: options
    pub fn add(&mut self, paths: &[&Path]) -> RequestResult<Vec<AddInfo>> {
        let res = self.request_multipart("add", paths.to_vec())?;
        let reader = BufReader::new(&res[..]);
        let mut infos = vec![];
        for info in reader.split(b'\n') {
            let add_info = serde_json::from_slice(&(info?)[..]);
            infos.push(add_info?);
        }
        Ok(infos)
    }

    // TODO: is this working? might need to specify encoding
    pub fn block_get<T: ToString>(&mut self, cid: T) -> RequestResult<String> {
        self.request_string_result("block/get", vec![cid])
    }

    pub fn cat<T: ToString>(&mut self, cid: T) -> RequestResult<hyper::Chunk> {
        self.request("cat", vec![cid])
    }

    pub fn commands(&mut self) -> RequestResult<CommandInfo> {
        let res = self.request::<_, String>("commands", vec![])?;
        Ok(serde_json::from_slice(&res)?)
    }

    pub fn config_get<T: ToString>(&mut self, key: T) -> RequestResult<String> {
        self.request_string_result("config", vec![key])
    }

    pub fn config_show(&mut self) -> RequestResult<String> {
        self.request_string_result::<_, String>("config/show", vec![])
    }

    pub fn id(&mut self) -> RequestResult<IdInfo> {
        let res = self.request::<_, String>("id", vec![])?;
        Ok(serde_json::from_slice(&res)?)
    }

    pub fn version(&mut self) -> RequestResult<VersionInfo> {
        let res = self.request::<_, String>("version", vec![])?;
        Ok(serde_json::from_slice(&res)?)
    }
}

#[derive(Debug)]
pub enum RequestError {
    HyperError(hyper::Error),
    IoError(io::Error),
    JsonError(serde_json::Error),
    Other(String),
}

impl From<hyper::Error> for RequestError {
    fn from(e: hyper::Error) -> RequestError {
        RequestError::HyperError(e)
    }
}

impl From<io::Error> for RequestError {
    fn from(e: io::Error) -> RequestError {
        RequestError::IoError(e)
    }
}

impl From<serde_json::Error> for RequestError {
    fn from(e: serde_json::Error) -> RequestError {
        RequestError::JsonError(e)
    }
}

impl From<multipart_legacy_client::RequestError> for RequestError {
    fn from(e: multipart_legacy_client::RequestError) -> RequestError {
        match e {
            multipart_legacy_client::RequestError::ParseError(e) =>
                RequestError::Other(format!("Parse error: {}", e)),
            multipart_legacy_client::RequestError::HyperError(e) =>
                RequestError::Other(format!("Hyper error: {}", e)),
            multipart_legacy_client::RequestError::IoError(e) =>
                RequestError::from(e),
            _ => RequestError::Other(format!("Error: {:?}", e)),
        }
    }
}

impl From<str::Utf8Error> for RequestError {
    fn from(e: str::Utf8Error) -> RequestError {
        RequestError::Other(format!("{:?}", e))
    }
}

#[cfg(test)]
mod tests {
    use multiaddr::{Multiaddr};

    #[test]
    fn thin_waist() {
        let test_maddrs = vec![
            "/ip4/127.0.0.1/udp/1234",
            "/ip4/127.0.0.1/tcp/1234",
            "/ip4/1.2.3.4",
            "/ip4/0.0.0.0",
            "/ip6/::1",
            "/ip6/2601:9:4f81:9700:803e:ca65:66e8:c21",
            "/ip6/2601:9:4f81:9700:803e:ca65:66e8:c21/udp/1234"
        ];

        for maddr_str in &test_maddrs {
            let maddr = match Multiaddr::new(maddr_str) {
                Err(e) => panic!("Error parsing multiaddr {}: {}", maddr_str, e),
                Ok(maddr) => maddr,
            };
            assert!(super::is_thin_waist(&maddr));
        }
    }
}
