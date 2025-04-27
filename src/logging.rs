use std::{fs::File, io::Write};
use wasm_bindgen::prelude::wasm_bindgen;

use crate::error::Error;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

struct Logger;
static LOGGER: Logger = Logger;

impl log::Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::Level::Trace
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let level = record.level();
            let level = match level {
                log::Level::Error => "error",
                log::Level::Warn => "warning",
                log::Level::Info => "notice",
                log::Level::Debug => "debug",
                log::Level::Trace => "debug",
            };
            let mut buf: Vec<u8> = Vec::new();
            let msg = record.args();
            write!(buf, "::{}::{}", level, msg).unwrap();
            log(&unsafe { String::from_utf8_unchecked(buf) });
        }
    }

    fn flush(&self) {}
}

pub fn init() {
    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(log::LevelFilter::Trace);
}

pub fn add_mask(value: &str) {
    log(&format!("::add-mask::{}", value));
}

pub fn set_output(name: &str, value: &str) -> Result<(), Error> {
    if let Ok(path) = std::env::var("GITHUB_OUTPUT") {
        let mut f = File::options()
            .append(true)
            .create(true)
            .open(path)
            .map_err(Error::new)?;
        writeln!(&mut f, "{}={}", name, value).map_err(Error::new)?;
    } else {
        log(&format!("::set-output name={}::{}", name, value));
    }
    Ok(())
}
