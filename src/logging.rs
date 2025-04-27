use std::io::Write;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{env, error::Error, fs};

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
    if let Some(path) = env::var("GITHUB_OUTPUT") {
        let data = format!("{}={}", name, value);
        // it maybe better to open the file at startup then buffer the writes
        fs::write_file_sync(
            &path,
            data.as_bytes(),
            &fs::FileWriteOptions {
                encoding: "utf8",
                mode: 0o666,
                flag: "a",
            },
        )?
    } else {
        log(&format!("::set-output name={}::{}", name, value));
    }
    Ok(())
}
