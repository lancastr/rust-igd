use std::io;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::str;
use std::time::Duration;

use futures::future;
use futures::{Future, IntoFuture, Stream};
use hyper;
use tokio::prelude::FutureExt;
use tokio::net::UdpSocket;
use xml::reader::XmlEvent;
use xml::EventReader;
use regex::Regex;

use crate::errors::SearchError;
use crate::Gateway;

// Content of the request.
pub const SEARCH_REQUEST: &'static str = "M-SEARCH * HTTP/1.1\r
Host:239.255.255.250:1900\r
ST:urn:schemas-upnp-org:device:InternetGatewayDevice:1\r
Man:\"ssdp:discover\"\r
MX:3\r\n\r\n";

/// Search gateway, bind to all interfaces and use a timeout of 3 seconds.
///
/// Bind to all interfaces.
/// The request will timeout after 3 seconds.
pub fn search_gateway() -> impl Future<Item = Gateway, Error = SearchError> {
    search_gateway_timeout(Duration::from_secs(3))
}

/// Search gateway, bind to all interfaces and use the given duration for the timeout.
///
/// Bind to all interfaces.
/// The request will timeout after the given duration.
pub fn search_gateway_timeout(timeout: Duration) -> impl Future<Item = Gateway, Error = SearchError> {
    search_gateway_from_timeout(Ipv4Addr::new(0, 0, 0, 0), timeout)
}

/// Search gateway, bind to the given interface and use a time of 3 seconds.
///
/// Bind to the given interface.
/// The request will timeout after 3 seconds.
pub fn search_gateway_from(ip: Ipv4Addr) -> impl Future<Item = Gateway, Error = SearchError> {
    search_gateway_from_timeout(ip, Duration::from_secs(3))
}

/// Search gateway, bind to the given interface and use the given duration for the timeout.
///
/// Bind to the given interface.
/// The request will timeout after the given duration.
pub fn search_gateway_from_timeout(ip: Ipv4Addr, timeout: Duration) -> impl Future<Item = Gateway, Error = SearchError> {
    let addr = SocketAddr::V4(SocketAddrV4::new(ip, 0));
    UdpSocket::bind(&addr)
        .into_future()
        .and_then(|socket| socket.send_dgram(SEARCH_REQUEST.as_bytes(), &"239.255.255.250:1900".parse().unwrap()))
        .and_then(|(socket, _)| socket.recv_dgram(vec![0u8; 1500]))
        .map_err(|err| SearchError::from(err))
        .and_then(|(_sock, buf, n, _addr)| {
            str::from_utf8(&buf[..n])
                .map_err(|err| SearchError::from(err))
                .and_then(|text| parse_result(text).ok_or(SearchError::InvalidResponse))
        })
        .and_then(move |location| get_control_url(&location).and_then(move |control_url| Ok(Gateway::new(location.0, control_url))))
        .timeout(timeout)
        .from_err()
}

fn get_control_url(location: &(SocketAddrV4, String)) -> Box<dyn Future<Item = String, Error = SearchError>> {
    let client = hyper::Client::new();
    let uri = match format!("http://{}{}", location.0, location.1).parse() {
        Ok(uri) => uri,
        Err(err) => return Box::new(future::err(SearchError::from(err))),
    };
    let future = client.get(uri).and_then(|resp| resp.into_body().concat2()).then(|result| match result {
        Ok(body) => parse_control_url(body.as_ref()),
        Err(err) => Err(SearchError::from(err)),
    });
    Box::new(future)
}

fn parse_control_url<R>(resp: R) -> Result<String, SearchError>
where
    R: io::Read,
{
    let parser = EventReader::new(resp);
    let mut chain = Vec::<String>::with_capacity(4);

    struct Service {
        service_type: String,
        control_url: String,
    }

    let mut service = Service {
        service_type: "".to_string(),
        control_url: "".to_string(),
    };

    for e in parser.into_iter() {
        match r#try!(e) {
            XmlEvent::StartElement { name, .. } => {
                chain.push(name.borrow().to_repr());
                let tail = if chain.len() >= 3 {
                    chain.iter().skip(chain.len() - 3)
                } else {
                    continue;
                };

                if vec!["device", "serviceList", "service"].iter().zip(tail).all(|(l, r)| l == r) {
                    service.service_type.clear();
                    service.control_url.clear();
                }
            }
            XmlEvent::EndElement { .. } => {
                let top = chain.pop();
                let tail = if top == Some("service".to_string()) && chain.len() >= 2 {
                    chain.iter().skip(chain.len() - 2)
                } else {
                    continue;
                };

                if vec!["device", "serviceList"].iter().zip(tail).all(|(l, r)| l == r)
                    && ("urn:schemas-upnp-org:service:WANIPConnection:1" == service.service_type || "urn:schemas-upnp-org:service:WANPPPConnection:1" == service.service_type)
                    && service.control_url.len() != 0
                {
                    return Ok(service.control_url);
                }
            }
            XmlEvent::Characters(text) => {
                let tail = if chain.len() >= 4 {
                    chain.iter().skip(chain.len() - 4)
                } else {
                    continue;
                };

                if vec!["device", "serviceList", "service", "serviceType"].iter().zip(tail.clone()).all(|(l, r)| l == r) {
                    service.service_type.push_str(&text);
                }
                if vec!["device", "serviceList", "service", "controlURL"].iter().zip(tail).all(|(l, r)| l == r) {
                    service.control_url.push_str(&text);
                }
            }
            _ => (),
        }
    }
    Err(SearchError::InvalidResponse)
}

// Parse the result.
pub fn parse_result(text: &str) -> Option<(SocketAddrV4, String)> {
    let re = Regex::new(
        r"(?i:Location):\s*http://(\d+\.\d+\.\d+\.\d+):(\d+)(/[^\r]*)",
    ).unwrap();
    for line in text.lines() {
        match re.captures(line) {
            None => continue,
            Some(cap) => {
                // these shouldn't fail if the regex matched.
                let addr = &cap[1];
                let port = &cap[2];
                return Some((
                    SocketAddrV4::new(
                        addr.parse::<Ipv4Addr>().unwrap(),
                        port.parse::<u16>().unwrap(),
                    ),
                    cap[3].to_string(),
                ));
            }
        }
    }
    None
}

mod tests {
    #[test]
    fn test_parse_result_case_insensitivity() {
        assert!(parse_result("location:http://0.0.0.0:0/control_url").is_some());
        assert!(parse_result("LOCATION:http://0.0.0.0:0/control_url").is_some());
    }

    #[test]
    fn test_parse_result() {
        let result = parse_result("location:http://0.0.0.0:0/control_url").unwrap();
        assert_eq!(result.0.ip(), &Ipv4Addr::new(0, 0, 0, 0));
        assert_eq!(result.0.port(), 0);
        assert_eq!(&result.1[..], "/control_url");
    }
}
