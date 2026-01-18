use std::{
    io::{Read, stdin},
    ops::Deref,
    process::ExitCode,
};

use roc_io_error::IOErr;
use roc_platform_builder::{
    RocArc, RocHost, RocSingleTagWrapper, RocUserData, host_fn, host_fn_try, platform_init,
};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

use roc_platform_builder::roc_std_new::{self as roc, RocList};

use roc::RocStr;
struct Host {
    s: String,
}

#[host_fn_try]
fn file_read_to_end(
    ops: &roc::RocOps,
    path: &RocStr,
) -> Result<RocStr, RocSingleTagWrapper<IOErr>> {
    let file_path = path.as_str();
    std::fs::read_to_string(file_path)
        .map(|s| RocStr::from_str(s.as_str(), ops))
        .map_err(|e| IOErr::from_io_error(&e, ops).into())
}

#[host_fn_try]
fn stdin_line(ops: &roc::RocOps) -> Result<RocStr, RocSingleTagWrapper<IOErr>> {
    let mut buf = String::with_capacity(1024);
    stdin()
        .read_line(&mut buf)
        .map(|_| RocStr::from_str(buf.as_str(), ops))
        .map_err(|e| IOErr::from_io_error(&e, ops).into())
}

#[host_fn]
fn stdin_read_to_end(ops: &roc::RocOps) -> Result<RocList<u8>, RocSingleTagWrapper<IOErr>> {
    let mut buf = Vec::with_capacity(1024);
    stdin()
        .read_to_end(&mut buf)
        .map(|_| RocList::from_slice(&buf, ops))
        .map_err(|e| IOErr::from_io_error(&e, ops).into())
}

#[host_fn]
fn stdout_get_some_type(_ops: &roc::RocOps) -> SomeType {
    SomeType::new("hi".into())
}

#[host_fn]
fn stdout_line(ops: &roc::RocOps, message: &RocStr) {
    println!("{} {}", message.as_str(), ops.user_data::<Host>().s);
}

type SomeType = RocArc<String>;

#[host_fn]
fn stdout_print_it(_ops: &roc::RocOps, some_type: &SomeType) {
    eprintln!("print_it: {}", some_type.deref())
}

#[host_fn]
fn stderr_line(_ops: &roc::RocOps, message: &RocStr) {
    eprintln!("{}", message.as_str());
}

fn init(args: &[String]) -> ExitCode {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();

    let host = RocHost::builder().build(Host {
        s: Default::default(),
    });

    host.run(args)
}

platform_init!(init);
