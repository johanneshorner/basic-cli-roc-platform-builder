use std::mem::MaybeUninit;

use reqwest::header::{HeaderName, HeaderValue};
use roc_platform_builder::roc_std_new::{
    RocList, RocOps, RocRefcounted, RocStr, roc_refcounted_noop_impl,
};

// TODO: Calling `dec()` on RocList/RocStr is a noop. How is this supposed to work?

#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
#[repr(u8)]
pub enum RequestErrTag {
    Builder = 0,
    Redirect = 1,
    Status = 2,
    Timeout = 3,
    Request = 4,
    Connect = 5,
    Body = 6,
    Decode = 7,
    Upgrade = 8,
    InvalidMethod = 9,
    InvalidHeaderName = 10,
    InvalidHeaderValue = 11,
    Other = 12,
}

impl core::fmt::Debug for RequestErrTag {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Builder => f.write_str("RequestErrTag::Builder"),
            Self::Redirect => f.write_str("RequestErrTag::Redirect"),
            Self::Status => f.write_str("RequestErrTag::Status"),
            Self::Timeout => f.write_str("RequestErrTag::Timeout"),
            Self::Request => f.write_str("RequestErrTag::Request"),
            Self::Connect => f.write_str("RequestErrTag::Connect"),
            Self::Body => f.write_str("RequestErrTag::Body"),
            Self::Decode => f.write_str("RequestErrTag::Decode"),
            Self::Upgrade => f.write_str("RequestErrTag::Upgrade"),
            Self::InvalidMethod => f.write_str("RequestErrTag::InvalidMethod"),
            Self::InvalidHeaderName => f.write_str("RequestErrTag::InvalidHeaderName"),
            Self::InvalidHeaderValue => f.write_str("RequestErrTag::InvalidHeaderValue"),
            Self::Other => f.write_str("RequestErrTag::Other"),
        }
    }
}

roc_refcounted_noop_impl!(RequestErrTag);

#[repr(C)]
#[derive(Debug, Clone)]
pub struct RequestErr {
    ///only valid for the `Status` variant (tag == 1)
    status_code: u16,
    pub tag: RequestErrTag,
}

impl RequestErr {
    pub fn invalid_method() -> Self {
        Self {
            status_code: 0,
            tag: RequestErrTag::InvalidMethod,
        }
    }

    pub fn invalid_header_name() -> Self {
        Self {
            status_code: 0,
            tag: RequestErrTag::InvalidHeaderName,
        }
    }

    pub fn invalid_header_value() -> Self {
        Self {
            status_code: 0,
            tag: RequestErrTag::InvalidHeaderValue,
        }
    }

    pub fn from_request_error(e: &reqwest::Error) -> Self {
        let simple = |tag| Self {
            status_code: 0,
            tag,
        };

        if e.is_builder() {
            simple(RequestErrTag::Builder)
        } else if e.is_redirect() {
            simple(RequestErrTag::Redirect)
        } else if e.is_status() {
            Self {
                status_code: e
                    .status()
                    .expect("a status error must have status code")
                    .into(),
                tag: RequestErrTag::Status,
            }
        } else if e.is_timeout() {
            simple(RequestErrTag::Timeout)
        } else if e.is_request() {
            simple(RequestErrTag::Request)
        } else if e.is_connect() {
            simple(RequestErrTag::Connect)
        } else if e.is_body() {
            simple(RequestErrTag::Body)
        } else if e.is_decode() {
            simple(RequestErrTag::Decode)
        } else if e.is_upgrade() {
            simple(RequestErrTag::Upgrade)
        } else {
            simple(RequestErrTag::Other)
        }
    }
}

impl From<reqwest::Error> for RequestErr {
    fn from(value: reqwest::Error) -> Self {
        Self::from_request_error(&value)
    }
}

roc_refcounted_noop_impl!(RequestErr);

#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
#[repr(u8)]
pub enum MethodTag {
    Options = 0,
    Get = 1,
    Post = 2,
    Put = 3,
    Delete = 4,
    Head = 5,
    Trace = 6,
    Connect = 7,
    Patch = 8,
    Extension = 9,
}

impl core::fmt::Debug for MethodTag {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Options => f.write_str("MethodTag::Options"),
            Self::Get => f.write_str("MethodTag::Get"),
            Self::Post => f.write_str("MethodTag::Post"),
            Self::Put => f.write_str("MethodTag::Put"),
            Self::Delete => f.write_str("MethodTag::Delete"),
            Self::Head => f.write_str("MethodTag::Head"),
            Self::Trace => f.write_str("MethodTag::Trace"),
            Self::Connect => f.write_str("MethodTag::Connect"),
            Self::Patch => f.write_str("MethodTag::Patch"),
            Self::Extension => f.write_str("MethodTag::Extension"),
        }
    }
}

roc_refcounted_noop_impl!(MethodTag);

#[repr(C)]
pub struct Method {
    extension: MaybeUninit<RocStr>,
    pub tag: MethodTag,
}

impl Method {
    pub fn extension(&self) -> Option<&RocStr> {
        if matches!(self.tag, MethodTag::Extension) {
            Some(unsafe { self.extension.assume_init_ref() })
        } else {
            None
        }
    }
}

impl Clone for Method {
    fn clone(&self) -> Self {
        if self.tag == MethodTag::Extension {
            Self {
                extension: MaybeUninit::new(unsafe { self.extension.assume_init_ref().clone() }),
                tag: self.tag,
            }
        } else {
            Self {
                extension: MaybeUninit::zeroed(),
                tag: self.tag,
            }
        }
    }
}

impl core::fmt::Debug for Method {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.tag == MethodTag::Extension {
            write!(f, "MethodTag::Extension({:?})", unsafe {
                self.extension.assume_init_ref()
            })
        } else {
            write!(f, "MethodTag::{:?}", self.tag)
        }
    }
}

impl RocRefcounted for Method {
    fn inc(&mut self) {
        if self.tag == MethodTag::Extension {
            unsafe { self.extension.assume_init_mut().inc() };
        }
    }
    fn dec(&mut self) {
        if self.tag == MethodTag::Extension {
            unsafe { self.extension.assume_init_mut().dec() };
        }
    }
    fn is_refcounted() -> bool {
        true
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Header {
    name: RocStr,
    value: RocStr,
}

impl TryFrom<&Header> for (HeaderName, HeaderValue) {
    type Error = RequestErr;

    fn try_from(Header { name, value }: &Header) -> Result<Self, Self::Error> {
        let name: HeaderName = name
            .as_str()
            .try_into()
            .map_err(|_| RequestErr::invalid_header_name())?;
        let value: HeaderValue = value
            .as_str()
            .try_into()
            .map_err(|_| RequestErr::invalid_header_value())?;
        Ok((name, value))
    }
}

impl RocRefcounted for Header {
    fn inc(&mut self) {
        self.name.inc();
        self.value.inc();
    }
    fn dec(&mut self) {
        self.name.dec();
        self.value.dec();
    }
    fn is_refcounted() -> bool {
        true
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Request {
    pub body: RocList<u8>,
    pub headers: RocList<Header>,
    pub method: Method,
    pub uri: RocStr,
}

impl RocRefcounted for Request {
    fn inc(&mut self) {
        self.method.inc();
        self.headers.inc();
        self.uri.inc();
        self.body.inc();
    }
    fn dec(&mut self) {
        self.method.dec();
        self.headers.dec();
        self.uri.dec();
        self.body.dec();
    }
    fn is_refcounted() -> bool {
        true
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Response {
    body: RocList<u8>,
    headers: RocList<Header>,
    status: u16,
}

impl Response {
    pub fn from_reqwest_response(response: reqwest::blocking::Response, ops: &RocOps) -> Self {
        // FIXME: return errors don't panic
        let status: u16 = response.status().into();
        let headers = response
            .headers()
            .into_iter()
            .map(|(name, value)| Header {
                name: RocStr::from_str(name.as_str(), ops),
                value: RocStr::from_str(value.to_str().expect("invalid header value"), ops),
            })
            .collect::<Vec<_>>();
        let body = RocList::from_slice(&response.bytes().expect("timeout"), ops);
        Self {
            status,
            headers: RocList::from_slice(&headers, ops),
            body,
        }
    }
}

impl RocRefcounted for Response {
    fn inc(&mut self) {
        self.headers.inc();
        self.body.inc();
    }
    fn dec(&mut self) {
        self.headers.dec();
        self.body.dec();
    }
    fn is_refcounted() -> bool {
        true
    }
}
