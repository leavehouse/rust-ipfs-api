extern crate futures;
extern crate hyper;
extern crate multiaddr;
extern crate multipart_legacy_client;
extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate serde_json;
extern crate tokio_core;

mod unmarshal;
mod util;

use futures::{Future, Stream};
use hyper::Chunk;
use multipart_legacy_client::send_new_post_request;
use tokio_core::reactor;
use std::str;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
pub use unmarshal::*;
pub use unmarshal::unmarshal;

// TODO: args could be an Iterator?
pub struct Request<'a, D> {
    api_base: String,
    command: &'a str,
    args: Vec<&'a str>,
    other_data: D,
}

impl<'a> Request<'a, ()> {
    pub fn new<'b>(cfg: &'b Config, command: &'a str, args: Vec<&'a str>)
                   -> Request<'a, ()> {
        Request::new_with_data(cfg, command, args, ())
    }
}

impl<'a, D> Request<'a, D> {
    pub fn new_with_data<'b>(cfg: &'b Config, command: &'a str, args: Vec<&'a str>,
                             other_data: D) -> Request<'a, D> {
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

    fn new_request<'a>(&self, command: &'a str, args: Vec<&'a str>)
                       -> Request<'a, ()> {
        Request::new(&self.config, command, args)
    }

    fn new_multipart_request<'a, 'b>(&self, command: &'a str, args: Vec<&'a str>,
                                     files: Vec<&'b Path>)
            -> Request<'a, Vec<&'b Path>> {
        Request::new_with_data(&self.config, command, args, files)
    }

    fn send_request(&mut self, request: &Request<()>)
                        -> RequestResult<Chunk> {
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
    fn send_request_multipart(&mut self, request: &Request<Vec<&Path>>)
                                  -> RequestResult<Vec<u8>> {
        let url = request.make_uri_string();
        Ok(send_new_post_request(url, &request.other_data[..])?)
    }

    fn request_string_result(&mut self, command: &str, args: Vec<&str>)
            -> RequestResult<String> {
        self.request(command, args)
            .map(|chunk| str::from_utf8(&chunk).unwrap().to_string())
    }


    fn request(&mut self, command: &str, args: Vec<&str>)
            -> RequestResult<Chunk> {
        let req = self.new_request(command, args);
        self.send_request(&req)
    }

    fn request_no_args(&mut self, command: &str) -> RequestResult<Chunk> {
        self.request(command, vec![])
    }

    fn request_multipart(&mut self, command: &str, files: Vec<&Path>)
            -> RequestResult<Vec<u8>> {
        let req = self.new_multipart_request(command, vec![], files);
        self.send_request_multipart(&req)
    }

    /*** start of API calls ***/
    // TODO: these should be on an HttpApi Trait, which would conform with
    // https://github.com/ipfs/interface-ipfs-core#api

    // TODO: options
    pub fn add(&mut self, paths: &[&Path]) -> RequestResult<Vec<Vec<u8>>> {
        let res = self.request_multipart("add", paths.to_vec())?;
        let reader = BufReader::new(&res[..]);
        let mut infos = vec![];
        // TODO: use something like reader.split(b'\n').map(|b| b?).collect()
        // problem is using ? in the closure argument to map.
        for info in reader.split(b'\n') {
            infos.push(info?);
        }
        Ok(infos)
    }

    pub fn bitswap_stat(&mut self) -> RequestResult<Chunk> {
        self.request_no_args("bitswap/stat")
    }

    // TODO: is this working? might need to specify encoding
    pub fn block_get<S: AsRef<str>>(&mut self, cid: S)
                                    -> RequestResult<String> {
        self.request_string_result("block/get", vec![cid.as_ref()])
    }

    pub fn bootstrap_list(&mut self) -> RequestResult<Chunk> {
        self.request_no_args("bootstrap/list")
    }

    pub fn cat<S: AsRef<str>>(&mut self, cid: S) -> RequestResult<Chunk> {
        self.request("cat", vec![cid.as_ref()])
    }

    pub fn commands(&mut self) -> RequestResult<Chunk> {
        self.request_no_args("commands")
    }

    pub fn config_get<S: AsRef<str>>(&mut self, key: S)
                                     -> RequestResult<String> {
        self.request_string_result("config", vec![key.as_ref()])
    }

    pub fn config_show(&mut self) -> RequestResult<String> {
        self.request_string_result("config/show", vec![])
    }

    pub fn id(&mut self) -> RequestResult<Chunk> {
        self.request_no_args("id")
    }

    pub fn log_ls(&mut self) -> RequestResult<Chunk> {
        self.request_no_args("log/ls")
    }

    // TODO: test that this keeps receiving chunks correctly?
    pub fn log_tail(&mut self) -> RequestResult<Chunk> {
        self.request_no_args("log/tail")
    }

    pub fn object_data<S: AsRef<str>>(&mut self, multihash: S)
                                      -> RequestResult<Chunk> {
        self.request("object/data", vec![multihash.as_ref()])
    }

    pub fn object_get<S: AsRef<str>>(&mut self, multihash: S)
                                     -> RequestResult<Chunk> {
        self.request("object/get", vec![multihash.as_ref()])
    }

    pub fn object_links<S: AsRef<str>>(&mut self, multihash: S)
                                       -> RequestResult<Chunk> {
        self.request("object/links", vec![multihash.as_ref()])
    }

    pub fn stats_bitswap(&mut self) -> RequestResult<Chunk> {
        self.request_no_args("stats/bitswap")
    }

    pub fn swarm_addrs(&mut self) -> RequestResult<Chunk> {
        self.request_no_args("swarm/addrs")
    }

    pub fn swarm_peers(&mut self) -> RequestResult<Chunk> {
        self.request_no_args("swarm/peers")
    }

    pub fn version(&mut self) -> RequestResult<Chunk> {
        self.request_no_args("version")
    }
}

#[derive(Debug)]
pub enum RequestError {
    HyperError(hyper::Error),
    IoError(io::Error),
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
