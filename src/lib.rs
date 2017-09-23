extern crate futures;
extern crate hyper;
extern crate multiaddr;
extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate serde_json;
extern crate tokio_core;


use futures::{Future, Stream};
use multiaddr::{Multiaddr, Protocol};
use tokio_core::reactor;
use std::{io, str};

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
pub struct Request {
    api_base: String,
    command: String,
    args: Vec<String>,
}

impl Request {
    pub fn new(cfg: &Config, command: String, args: Vec<String>) -> Request {
        let api_base = format!("http://{}:{}{}", cfg.host, cfg.port,
                               cfg.api_path);
        Request { api_base, command, args }
    }

    // TODO: options
    pub fn getUri(&self) -> hyper::Uri {
        let args_str = self.args.iter()
                                .map(|ref arg| format!("arg={}", arg))
                                .collect::<Vec<_>>()
                                .join("&");
        let uri_string = format!("{}/{}?{}", self.api_base, self.command, args_str);
        match uri_string.parse() {
            Err(err) => panic!("Parse of {} failed: {}", uri_string, err),
            Ok(uri) => uri,
        }
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

pub struct IpfsApi {
    config: Config,
    core: reactor::Core,
    client: hyper::Client<hyper::client::HttpConnector>,
}

#[derive(Debug, Deserialize)]
pub struct VersionInfo {
    Version: String,
    Commit: String,
    Repo: String,
    System: String,
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

fn chunk_to_string(ch: &hyper::Chunk) -> String {
    str::from_utf8(ch).unwrap().to_string()
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

    pub fn new_request(&self, command: String, args: Vec<String>) -> Request {
        Request::new(&self.config, command, args)
    }

    pub fn send_request(&mut self, request: &Request) -> RequestResult<hyper::Chunk> {
        let uri = request.getUri();
        let hyper_req = hyper::Request::new(hyper::Method::Post, uri);
        //req.headers_mut().set(ContentType::json());
        //req.headers_mut().set(ContentLength(json.len() as u64));
        //hyper_req.set_body(request);
        let post = self.client.request(hyper_req).and_then(|res| {
            res.body().concat2()
        });

        Ok(self.core.run(post)?)
    }

    fn request_string_result<T, U>(&mut self, command: T, args: Vec<U>) -> RequestResult<String>
                                   where T: ToString,
                                         U: ToString {
        self.request(command, args)
            .map(|chunk| str::from_utf8(&chunk).unwrap().to_string())
    }


    pub fn request<T,U>(&mut self, command: T, args: Vec<U>) -> RequestResult<hyper::Chunk>
                        where T: ToString, U: ToString {
        let req = self.new_request(command.to_string(),
                                   args.into_iter().map(|a| a.to_string()).collect());
        self.send_request(&req)
    }

    // TODO: is this working? might need to specify encoding
    pub fn block_get<T: ToString>(&mut self, cid: T) -> RequestResult<String> {
        self.request_string_result("block/get", vec![cid])
    }

    pub fn cat<T: ToString>(&mut self, cid: T) -> RequestResult<String> {
        self.request_string_result("cat", vec![cid])
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
