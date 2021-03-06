// Copyright 2016 LambdaStack All rights reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![allow(dead_code)]

use std::{io, slice, str};
use std::fmt::{self,Write};
use std::str::FromStr;

use unicase::UniCase;
use http::date;
use Body;
use Headers;
use StatusCode;

#[derive(Clone, Debug)]
pub struct Response {
    pub headers: Headers,
    pub body: Body,
    pub status_message: StatusMessage,
    pub code: u16,
    pub message: String,
}

#[derive(Clone, Debug)]
pub enum StatusMessage {
    Ok,
    Custom(u16, String)
}

impl Response {
    pub fn new() -> Response {
        let status = StatusCode::Ok;

        let res = Response {
            headers: Headers::new(),
            body: Body::new(),
            status_message: StatusMessage::Custom(status.to_u16(), status.canonical_reason().unwrap_or("").to_string()),
            code: status.to_u16(),
            message: status.canonical_reason().unwrap_or("").to_string(),
        };

        res
    }

    #[inline]
    pub fn with_body(mut self, body: Body) -> Self { //&mut Response {
        self.body = body;
        self
    }

    #[inline]
    pub fn with_header(mut self, name: &str, val: &str) -> Self {
        self.headers.push((name.to_string(), val.to_string()));
        self
    }

    #[inline]
    pub fn with_status(mut self, code: StatusCode) -> Self {
        self.code = code.to_u16();
        self.message = code.canonical_reason().clone().unwrap_or("").to_string();
        self.status_message = StatusMessage::Custom(code.to_u16(), code.canonical_reason().unwrap_or("").to_string());
        self
    }

    pub fn status_code(mut self, code: u16, message: &str) -> Self {
        self.code = code;
        self.message = message.clone().to_string();
        self.status_message = StatusMessage::Custom(code, message.to_string());
        self
    }

    pub fn content_length(&self) -> u64 {
        match self.header("content-length") {
            Some(len) => len.parse::<u64>().unwrap_or(0),
            None => 0,
        }
    }

    pub fn header(&self, key: &str) -> Option<&str> {
        match self.headers.iter().find(|&&(ref k, ref v)| UniCase(k) == UniCase(key)) {
            Some(&(ref key, ref value)) => Some(str::from_utf8(value.as_bytes()).unwrap_or("")),
            None => None
        }
    }
}

// NOTE: May want to modify this to a different header write option...

pub fn encode(res: &Response, buf: &mut Vec<u8>) {
    let length = res.body.len();
    let now = date::now();

    write!(FastWrite(buf), "\
        HTTP/1.1 {}\r\n\
        Date: {}\r\n\
    ", res.status_message, now).unwrap();

    for &(ref k, ref v) in &res.headers {
        buf.extend_from_slice(k.as_bytes());
        buf.extend_from_slice(b": ");
        buf.extend_from_slice(v.as_bytes());
        buf.extend_from_slice(b"\r\n");
    }

    buf.extend_from_slice(b"\r\n");
    buf.extend_from_slice(&res.body[..]); //.as_bytes());
}

// TODO: impl fmt::Write for Vec<u8>
//
// Right now `write!` on `Vec<u8>` goes through io::Write and is not super
// speedy, so inline a less-crufty implementation here which doesn't go through
// io::Error.
struct FastWrite<'a>(&'a mut Vec<u8>);

impl<'a> fmt::Write for FastWrite<'a> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.0.extend_from_slice(s.as_bytes());
        Ok(())
    }

    fn write_fmt(&mut self, args: fmt::Arguments) -> fmt::Result {
        fmt::write(self, args)
    }
}

impl fmt::Display for StatusMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            StatusMessage::Ok => f.pad("200 OK"),
            StatusMessage::Custom(c, ref s) => write!(f, "{} {}", c, s),
        }
    }
}
