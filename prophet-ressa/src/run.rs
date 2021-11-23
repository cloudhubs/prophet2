use std::net::{SocketAddr, ToSocketAddrs};

use crate::extract_ressas;
use actix_web::{client::Client, http::Method};
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use source_code_parser::{Directory, ModuleComponent, ressa::NodePattern};
use structopt::StructOpt;

use derive_new::new;

#[derive(StructOpt, Debug)]
struct Opt {
    #[structopt(long, short, default_value = "127.0.0.1")]
    bounded_context_host: String,
    #[structopt(long, short, default_value = "8080")]
    bounded_context_port: u16,
}

pub enum Error {
    IOError(std::io::Error),
    MinifyError(prophet_ressa_minify::MinifyError),
    HTTPError(actix_web::client::SendRequestError),
    DeserializationError(actix_web::client::JsonPayloadError),
}

#[derive(Serialize, new)]
pub struct Request {
    /// Directory to search
    dir: Directory,

    /// ReSSAs to send to the server
    ressas: Vec<NodePattern>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    // TODO fill fields
}

/// Connection options
static ADDRESS: OnceCell<SocketAddr> = OnceCell::new();

/// URL on the server for the API
static URL: &'static str = "/ressa";

pub async fn run_ressa_parse(ast: &Vec<ModuleComponent>, ressa_dir: &str) -> Result<(), Error> {
    let ressas = extract_ressas(ast, ressa_dir)?;

    // Validate address
    let address = ADDRESS.get_or_init(|| {
        let opts = Opt::from_args();
        match (opts.bounded_context_host.clone(), opts.bounded_context_port).to_socket_addrs() {
            Ok(mut addrs) => match addrs.next() {
                Some(addr) => addr,
                None => panic!("No address resolves to {:?}", opts),
            },
            Err(err) => panic!("Invalid options {:?}: {:#?}", opts, err),
        }
    });

    // Make request
    //
    

    // Parse request
    let result = match response {
        Ok(mut resp) => match resp.json::<Response>().await {
            Ok(data) => data,
            Err(err) => return Err(Error::DeserializationError(err)),
        },
        Err(err) => return Err(Error::HTTPError(err)),
    };

    // result.into()
    Ok(())
}
