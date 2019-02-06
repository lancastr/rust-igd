use std::io;
use std::string::FromUtf8Error;

use futures::future;
use futures::{Future, Stream};
use hyper;
use hyper::error::Error as HyperError;
use hyper::header::{CONTENT_LENGTH, CONTENT_TYPE};
use hyper::{Client, Request};
use hyper::http::uri::InvalidUri;
use mime::TEXT_XML;

pub enum Error {
    HttpError(HyperError),
    IoError(io::Error),
    InvalidUri(InvalidUri),
    Utf8Error(FromUtf8Error),
}

impl From<HyperError> for Error {
    fn from(err: HyperError) -> Error {
        Error::HttpError(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::IoError(err)
    }
}

impl From<InvalidUri> for Error {
    fn from(err: InvalidUri) -> Error {
        Error::InvalidUri(err)
    }
}

impl From<FromUtf8Error> for Error {
    fn from(err: FromUtf8Error) -> Error {
        Error::Utf8Error(err)
    }
}

pub fn send_async(url: &str, action: &str, body: &str) -> Box<Future<Item = String, Error = Error>> {
    let client = Client::new();
    let uri: hyper::Uri = match url.parse() {
        Ok(uri) => uri,
        Err(err) => return Box::new(future::err(Error::from(err))),
    };
    let mut req = Request::builder();
    let req = req
        .method("POST")
        .uri(uri)
        .header("SOAPAction", action)
        .header(CONTENT_TYPE, TEXT_XML.as_ref())
        .header(CONTENT_LENGTH, body.len())
        .body(hyper::Body::from(body.to_owned())).unwrap();

    let future = client
        .request(req)
        .and_then(|resp| resp.into_body().concat2())
        .map_err(|err| Error::from(err))
        .and_then(|bytes| String::from_utf8(bytes.to_vec()).map_err(|err| Error::from(err)));
    Box::new(future)
}
