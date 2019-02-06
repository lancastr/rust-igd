use std;
use std::io;
use std::str;

use hyper;
use xml::reader::Error as XmlError;

use crate::soap;
use tokio;
use std::string::FromUtf8Error;

/// Errors that can occur when sending the request to the gateway.
#[derive(Debug, Fail)]
pub enum RequestError {
    /// Http/Hyper error
    #[fail(display = "HTTP error: _0")]
    HttpError(hyper::Error),
    /// IO Error
    #[fail(display = "IO error: _0")]
    IoError(io::Error),
    /// The response from the gateway could not be parsed.
    #[fail(display = "Invalid response from gateway: _0")]
    InvalidResponse(String),
    /// The gateway returned an unhandled error code and description.
    #[fail(display = "Gateway response error _0: _1")]
    ErrorCode(u16, String),
    /// Tokio timer error
    #[fail(display = "The port was not mapped")]
    TimerError(tokio::timer::Error),
    /// UTF-8 decoding error
    #[fail(display = "UTF-8 error: _0")]
    Utf8Error(FromUtf8Error),
    /// Invalid URI
    #[fail(display = "Invalid URI error: _0")]
    InvalidUri(hyper::http::uri::InvalidUri),
}

impl From<tokio::timer::Error> for RequestError {
    fn from(e: tokio::timer::Error) -> Self {
        RequestError::TimerError(e)
    }
}
/// Errors returned by `Gateway::get_external_ip`
#[derive(Debug, Fail)]
pub enum GetExternalIpError {
    /// The client is not authorized to perform the operation.
    #[fail(display = "The client is not authorized to remove the port")]
    ActionNotAuthorized,
    /// Some other error occured performing the request.
    #[fail(display = "Request Error: _0")]
    RequestError(RequestError),
}

/// Errors returned by `Gateway::remove_port`
#[derive(Debug, Fail)]
pub enum RemovePortError {
    /// The client is not authorized to perform the operation.
    #[fail(display = "The client is not authorized to remove the port")]
    ActionNotAuthorized,
    /// No such port mapping.
    #[fail(display = "The port was not mapped")]
    NoSuchPortMapping,
    /// Some other error occured performing the request.
    #[fail(display = "Request error. _0")]
    RequestError(RequestError),
}

/// Errors returned by `Gateway::add_any_port` and `Gateway::get_any_address`
#[derive(Debug, Fail)]
pub enum AddAnyPortError {
    /// The client is not authorized to perform the operation.
    #[fail(display = "The client is not authorized to remove the port")]
    ActionNotAuthorized,
    /// Can not add a mapping for local port 0.
    #[fail(display = "Can not add a mapping for local port 0")]
    InternalPortZeroInvalid,
    /// The gateway does not have any free ports.
    #[fail(display = "The gateway does not have any free ports")]
    NoPortsAvailable,
    /// The gateway can only map internal ports to same-numbered external ports
    /// and this external port is in use.
    #[fail(display = "The gateway can only map internal ports to same-numbered external ports and this external port is in use.")]
    ExternalPortInUse,
    /// The gateway only supports permanent leases (ie. a `lease_duration` of 0).
    #[fail(display = "The gateway only supports permanent leases (ie. a `lease_duration` of 0),")]
    OnlyPermanentLeasesSupported,
    /// The description was too long for the gateway to handle.
    #[fail(display = "The description was too long for the gateway to handle.")]
    DescriptionTooLong,
    /// Some other error occured performing the request.
    #[fail(display = "Request error. _0")]
    RequestError(RequestError),
}

impl From<RequestError> for AddAnyPortError {
    fn from(err: RequestError) -> AddAnyPortError {
        AddAnyPortError::RequestError(err)
    }
}

/// Errors returned by `Gateway::add_port`
#[derive(Debug, Fail)]
pub enum AddPortError {
    /// The client is not authorized to perform the operation.
    #[fail(display = "The client is not authorized to map this port.")]
    ActionNotAuthorized,
    /// Can not add a mapping for local port 0.
    #[fail(display = "Can not add a mapping for local port 0")]
    InternalPortZeroInvalid,
    /// External port number 0 (any port) is considered invalid by the gateway.
    #[fail(display = "External port number 0 (any port) is considered invalid by the gateway.")]
    ExternalPortZeroInvalid,
    /// The requested mapping conflicts with a mapping assigned to another client.
    #[fail(display = "The requested mapping conflicts with a mapping assigned to another client.")]
    PortInUse,
    /// The gateway requires that the requested internal and external ports are the same.
    #[fail(display = "The gateway requires that the requested internal and external ports are the same.")]
    SamePortValuesRequired,
    /// The gateway only supports permanent leases (ie. a `lease_duration` of 0).
    #[fail(display = "The gateway only supports permanent leases (ie. a `lease_duration` of 0),")]
    OnlyPermanentLeasesSupported,
    /// The description was too long for the gateway to handle.
    #[fail(display = "The description was too long for the gateway to handle.")]
    DescriptionTooLong,
    /// Some other error occured performing the request.
    #[fail(display = "Request error. _0")]
    RequestError(RequestError),
}

impl From<io::Error> for RequestError {
    fn from(err: io::Error) -> RequestError {
        RequestError::IoError(err)
    }
}

impl From<soap::Error> for RequestError {
    fn from(err: soap::Error) -> RequestError {
        match err {
            soap::Error::HttpError(e) => RequestError::HttpError(e),
            soap::Error::IoError(e) => RequestError::IoError(e),
            soap::Error::InvalidUri(e) => RequestError::InvalidUri(e),
            soap::Error::Utf8Error(e) => RequestError::Utf8Error(e),
        }
    }
}

/// Errors than can occur while trying to find the gateway.
#[derive(Debug, Fail)]
pub enum SearchError {
    /// Http/Hyper error
    #[fail(display = "HTTP error: _0")]
    HttpError(hyper::Error),
    /// Unable to process the response
    #[fail(display = "Invalid URI: _0")]
    InvalidUri(hyper::http::uri::InvalidUri),
    /// The response from the gateway could not be parsed.
    #[fail(display = "Invalid response")]
    InvalidResponse,
    /// IO Error
    #[fail(display = "IO error: _0")]
    IoError(io::Error),
    /// UTF-8 decoding error
    #[fail(display = "UTF-8 error: _0")]
    Utf8Error(str::Utf8Error),
    /// XML processing error
    #[fail(display = "XML error: _0")]
    XmlError(XmlError),
}

impl From<hyper::http::uri::InvalidUri> for SearchError {
    fn from(e: hyper::http::uri::InvalidUri) -> Self {
        SearchError::InvalidUri(e)
    }
}

impl From<hyper::Error> for SearchError {
    fn from(err: hyper::Error) -> SearchError {
        SearchError::HttpError(err)
    }
}

impl From<io::Error> for SearchError {
    fn from(err: io::Error) -> SearchError {
        SearchError::IoError(err)
    }
}

impl From<str::Utf8Error> for SearchError {
    fn from(err: str::Utf8Error) -> SearchError {
        SearchError::Utf8Error(err)
    }
}

impl From<XmlError> for SearchError {
    fn from(err: XmlError) -> SearchError {
        SearchError::XmlError(err)
    }
}

impl From<tokio::timer::timeout::Error<SearchError>> for SearchError {
    fn from(_err: tokio::timer::timeout::Error<SearchError>) -> SearchError {
        SearchError::IoError(io::Error::new(io::ErrorKind::TimedOut, "search timed out"))
    }
}
