use std::{
    io::{Read, stdin},
    mem::ManuallyDrop,
    ops::Deref,
    process::ExitCode,
};

use roc_command::CommandOutputSuccess;
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
fn cmd_exec_exit_code(
    ops: &roc::RocOps,
    cmd: &roc_command::Command,
) -> Result<i32, RocSingleTagWrapper<IOErr>> {
    roc_command::command_exec_exit_code(cmd, ops).map_err(|e| e.into())
}

#[repr(C)]
pub struct NonZeroExitPayload {
    pub stderr_utf8_lossy: RocStr, // offset 0 (24 bytes)
    pub stdout_utf8_lossy: RocStr, // offset 24 (24 bytes)
    pub exit_code: i32,            // offset 48 (4 bytes + padding)
}

#[repr(C)]
pub union CmdOutputErrPayload {
    cmd_err: ManuallyDrop<roc_io_error::IOErr>,
    non_zero_exit: ManuallyDrop<NonZeroExitPayload>,
}

#[repr(C)]
pub struct CmdOutputErr {
    payload: CmdOutputErrPayload,
    discriminant: u8, // CmdErr=0, NonZeroExit=1
}

impl CmdOutputErr {
    pub fn cmd_err(io_err: roc_io_error::IOErr) -> Self {
        Self {
            payload: CmdOutputErrPayload {
                cmd_err: core::mem::ManuallyDrop::new(io_err),
            },
            discriminant: 0,
        }
    }

    pub fn non_zero_exit(
        stderr_utf8_lossy: RocStr,
        stdout_utf8_lossy: RocStr,
        exit_code: i32,
    ) -> Self {
        Self {
            payload: CmdOutputErrPayload {
                non_zero_exit: core::mem::ManuallyDrop::new(NonZeroExitPayload {
                    stderr_utf8_lossy,
                    stdout_utf8_lossy,
                    exit_code,
                }),
            },
            discriminant: 1,
        }
    }
}

#[host_fn_try]
fn cmd_exec_output(
    ops: &roc::RocOps,
    cmd: &roc_command::Command,
) -> Result<CommandOutputSuccess, CmdOutputErr> {
    match roc_command::command_exec_output(cmd, ops) {
        roc_command::CommandOutputResult::Success(output) => Ok(CommandOutputSuccess {
            stderr_utf8_lossy: output.stderr_utf8_lossy,
            stdout_utf8: output.stdout_utf8,
        }),
        roc_command::CommandOutputResult::NonZeroExit(failure) => Err(CmdOutputErr::non_zero_exit(
            failure.stderr_utf8_lossy,
            failure.stdout_utf8_lossy,
            failure.exit_code,
        )),
        roc_command::CommandOutputResult::Error(io_err) => Err(CmdOutputErr::cmd_err(io_err)),
    }
}

#[host_fn_try]
fn dir_create(ops: &roc::RocOps, path: &RocStr) -> Result<(), RocSingleTagWrapper<IOErr>> {
    std::fs::create_dir(path.as_str()).map_err(|e| IOErr::from_io_error(&e, ops).into())
}

#[host_fn_try]
fn dir_create_all(ops: &roc::RocOps, path: &RocStr) -> Result<(), RocSingleTagWrapper<IOErr>> {
    std::fs::create_dir_all(path.as_str()).map_err(|e| IOErr::from_io_error(&e, ops).into())
}

#[host_fn_try]
fn dir_delete_empty(ops: &roc::RocOps, path: &RocStr) -> Result<(), RocSingleTagWrapper<IOErr>> {
    std::fs::remove_dir(path.as_str()).map_err(|e| IOErr::from_io_error(&e, ops).into())
}

#[host_fn_try]
fn dir_delete_all(ops: &roc::RocOps, path: &RocStr) -> Result<(), RocSingleTagWrapper<IOErr>> {
    std::fs::remove_dir_all(path.as_str()).map_err(|e| IOErr::from_io_error(&e, ops).into())
}

#[host_fn_try]
fn dir_list(
    ops: &roc::RocOps,
    path: &RocStr,
) -> Result<RocList<RocStr>, RocSingleTagWrapper<IOErr>> {
    std::fs::read_dir(path.as_str())
        .map(|read_dir| {
            let entries: Vec<_> = read_dir
                .filter_map(|entry| {
                    entry
                        .ok()
                        .map(|e| RocStr::from_str(&e.path().to_string_lossy(), ops))
                })
                .collect();
            RocList::from_slice(&entries, ops)
        })
        .map_err(|e| IOErr::from_io_error(&e, ops).into())
}

#[host_fn]
fn env_var(ops: &roc::RocOps, name: &RocStr) -> RocStr {
    let value = std::env::var(name.as_str()).unwrap_or_default();
    RocStr::from_str(&value, ops)
}

#[host_fn]
fn env_cwd(ops: &roc::RocOps) -> RocStr {
    let cwd = std::env::current_dir()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default();
    RocStr::from_str(&cwd, ops)
}

#[host_fn]
fn env_exe_path(ops: &roc::RocOps) -> RocStr {
    let exe_path = std::env::current_exe()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default();
    RocStr::from_str(&exe_path, ops)
}

#[host_fn_try]
fn file_read_bytes(
    ops: &roc::RocOps,
    path: &RocStr,
) -> Result<RocList<u8>, RocSingleTagWrapper<IOErr>> {
    std::fs::read(path.as_str())
        .map(|s| RocList::from_slice(&s, ops))
        .map_err(|e| IOErr::from_io_error(&e, ops).into())
}

#[host_fn_try]
fn file_write_bytes(
    ops: &roc::RocOps,
    path: &RocStr,
    bytes: &RocList<u8>,
) -> Result<(), RocSingleTagWrapper<IOErr>> {
    std::fs::write(path.as_str(), bytes.as_slice())
        .map_err(|e| IOErr::from_io_error(&e, ops).into())
}

#[host_fn_try]
fn file_read_utf8(ops: &roc::RocOps, path: &RocStr) -> Result<RocStr, RocSingleTagWrapper<IOErr>> {
    std::fs::read_to_string(path.as_str())
        .map(|s| RocStr::from_str(s.as_str(), ops))
        .map_err(|e| IOErr::from_io_error(&e, ops).into())
}

#[host_fn_try]
fn file_write_utf8(
    ops: &roc::RocOps,
    path: &RocStr,
    s: &RocStr,
) -> Result<(), RocSingleTagWrapper<IOErr>> {
    std::fs::write(path.as_str(), s.as_str()).map_err(|e| IOErr::from_io_error(&e, ops).into())
}

#[host_fn_try]
fn file_delete(ops: &roc::RocOps, path: &RocStr) -> Result<(), RocSingleTagWrapper<IOErr>> {
    std::fs::remove_file(path.as_str()).map_err(|e| IOErr::from_io_error(&e, ops).into())
}

#[host_fn_try]
fn path_is_file(ops: &roc::RocOps, path: &RocStr) -> Result<bool, RocSingleTagWrapper<IOErr>> {
    std::path::Path::new(path.as_str())
        .symlink_metadata()
        .map(|m| m.is_file())
        .map_err(|e| IOErr::from_io_error(&e, ops).into())
}

#[host_fn_try]
fn path_is_dir(ops: &roc::RocOps, path: &RocStr) -> Result<bool, RocSingleTagWrapper<IOErr>> {
    std::path::Path::new(path.as_str())
        .symlink_metadata()
        .map(|m| m.is_dir())
        .map_err(|e| IOErr::from_io_error(&e, ops).into())
}

#[host_fn_try]
fn path_is_sym_link(ops: &roc::RocOps, path: &RocStr) -> Result<bool, RocSingleTagWrapper<IOErr>> {
    std::path::Path::new(path.as_str())
        .symlink_metadata()
        .map(|m| m.is_symlink())
        .map_err(|e| IOErr::from_io_error(&e, ops).into())
}

#[host_fn_try]
fn random_seed_u64(ops: &roc::RocOps) -> Result<u64, RocSingleTagWrapper<IOErr>> {
    Ok(roc_random::random_u64(ops)?)
}

#[host_fn_try]
fn random_seed_u32(ops: &roc::RocOps) -> Result<u32, RocSingleTagWrapper<IOErr>> {
    Ok(roc_random::random_u32(ops)?)
}

#[host_fn]
fn sleep_millis(_ops: &roc::RocOps, millis: &u64) {
    std::thread::sleep(std::time::Duration::from_millis(*millis));
}

#[host_fn]
fn stderr_line(_ops: &roc::RocOps, message: &RocStr) {
    eprintln!("{}", message.as_str());
}

#[host_fn]
fn stderr_write(_ops: &roc::RocOps, message: &RocStr) {
    eprint!("{}", message.as_str());
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
