use chrono::{
    format::{DelayedFormat, StrftimeItems},
    Local,
};
use std::fmt::{Arguments, Display, Write};

pub struct DebugBuf {
    buf: String,
}

impl Write for DebugBuf {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.buf.write_str(s)
    }

    fn write_char(&mut self, c: char) -> std::fmt::Result {
        self.buf.write_char(c)
    }

    fn write_fmt(&mut self, args: Arguments<'_>) -> std::fmt::Result {
        self.buf.write_fmt(args)
    }
}

impl DebugBuf {
    pub fn new() -> DebugBuf {
        let now = display_now();
        DebugBuf { buf: format!("{now}: ") }
    }

    pub fn write_str(mut self, s: &str) -> Self {
        self.buf.write_str(s).unwrap();
        self
    }

    pub fn write_char(mut self, c: char) -> Self {
        self.buf.write_char(c).unwrap();
        self
    }

    pub fn write_fmt(mut self, args: Arguments<'_>) -> Self {
        self.buf.write_fmt(args).unwrap();
        self
    }

    pub fn axum_req<T>(self, req: &axum::http::Request<T>) -> Self {
        let method = req.method();
        let host = req.headers().get("host").unwrap().to_str().unwrap();
        let uri = req.uri();
        self.write_fmt(format_args!("{method} \"{host}{uri}\""))
    }

    pub fn axum_req_with_scheme<T>(self, req: &axum::http::Request<T>, scheme: &str) -> Self {
        let method = req.method();
        let host = req.headers().get("host").unwrap().to_str().unwrap();
        let uri = req.uri();
        self.write_fmt(format_args!("{method} \"{scheme}://{host}{uri}\""))
    }

    pub fn to(self) -> Self {
        self.write_str(" -> ")
    }

    pub fn val(self, val: impl Display) -> Self {
        self.write_fmt(format_args!("{val}"))
    }

    pub fn reqwest_req_mut(&mut self, req: &reqwest::Request) {
        let method = req.method();
        let uri = req.url();
        self.write_fmt(format_args!("{method} \"{uri}\"")).unwrap();
    }

    pub fn reqwest_req(mut self, req: &reqwest::Request) -> Self {
        self.reqwest_req_mut(req);
        self
    }

    pub fn not_found(self) -> Self {
        self.write_str("404 NOT FOUND")
    }

    pub fn debugln(&self) {
        eprintln!("[DEBUG] {}", self.buf)
    }

    pub fn infoln(&self) {
        eprintln!("[ INFO] {}", self.buf)
    }

    pub fn warnln(&self) {
        eprintln!("[ WARN] {}", self.buf)
    }

    pub fn errorln(&self) {
        eprintln!("[ERROR] {}", self.buf)
    }
}

pub fn display_now() -> DelayedFormat<StrftimeItems<'static>> {
    Local::now().format("%Y-%m-%d %H:%M:%S.%3f")
}
