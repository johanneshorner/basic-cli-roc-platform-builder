use std::{
    io::{Read, stdin},
    mem::ManuallyDrop,
    process::ExitCode,
};

use roc_command::CommandOutputSuccess;
use roc_io_error::IOErr;
use roc_platform_builder::{RocHost, RocSingleTagWrapper, host, platform_init};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

use roc_platform_builder::roc_std_new::{self as roc, RocList};

use roc::RocStr;

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

struct Host;

#[host]
impl Host {
    #[fallible]
    fn cmd_exec_exit_code(
        &mut self,
        ops: &roc::RocOps,
        cmd: &roc_command::Command,
    ) -> Result<i32, RocSingleTagWrapper<IOErr>> {
        roc_command::command_exec_exit_code(cmd, ops).map_err(|e| e.into())
    }

    #[fallible]
    fn cmd_exec_output(
        &mut self,
        ops: &roc::RocOps,
        cmd: &roc_command::Command,
    ) -> Result<CommandOutputSuccess, CmdOutputErr> {
        match roc_command::command_exec_output(cmd, ops) {
            roc_command::CommandOutputResult::Success(output) => Ok(CommandOutputSuccess {
                stderr_utf8_lossy: output.stderr_utf8_lossy,
                stdout_utf8: output.stdout_utf8,
            }),
            roc_command::CommandOutputResult::NonZeroExit(failure) => {
                Err(CmdOutputErr::non_zero_exit(
                    failure.stderr_utf8_lossy,
                    failure.stdout_utf8_lossy,
                    failure.exit_code,
                ))
            }
            roc_command::CommandOutputResult::Error(io_err) => Err(CmdOutputErr::cmd_err(io_err)),
        }
    }

    #[fallible]
    fn dir_create(
        &mut self,
        ops: &roc::RocOps,
        path: &RocStr,
    ) -> Result<(), RocSingleTagWrapper<IOErr>> {
        std::fs::create_dir(path.as_str()).map_err(|e| IOErr::from_io_error(&e, ops).into())
    }

    #[fallible]
    fn dir_create_all(
        &mut self,
        ops: &roc::RocOps,
        path: &RocStr,
    ) -> Result<(), RocSingleTagWrapper<IOErr>> {
        std::fs::create_dir_all(path.as_str()).map_err(|e| IOErr::from_io_error(&e, ops).into())
    }

    #[fallible]
    fn dir_delete_empty(
        &mut self,
        ops: &roc::RocOps,
        path: &RocStr,
    ) -> Result<(), RocSingleTagWrapper<IOErr>> {
        std::fs::remove_dir(path.as_str()).map_err(|e| IOErr::from_io_error(&e, ops).into())
    }

    #[fallible]
    fn dir_delete_all(
        &mut self,
        ops: &roc::RocOps,
        path: &RocStr,
    ) -> Result<(), RocSingleTagWrapper<IOErr>> {
        std::fs::remove_dir_all(path.as_str()).map_err(|e| IOErr::from_io_error(&e, ops).into())
    }

    #[fallible]
    fn dir_list(
        &mut self,
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

    fn env_var(&mut self, ops: &roc::RocOps, name: &RocStr) -> RocStr {
        let value = std::env::var(name.as_str()).unwrap_or_default();
        RocStr::from_str(&value, ops)
    }

    fn env_cwd(&mut self, ops: &roc::RocOps) -> RocStr {
        let cwd = std::env::current_dir()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_default();
        RocStr::from_str(&cwd, ops)
    }

    fn env_exe_path(&mut self, ops: &roc::RocOps) -> RocStr {
        let exe_path = std::env::current_exe()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_default();
        RocStr::from_str(&exe_path, ops)
    }

    #[fallible]
    fn file_read_bytes(
        &mut self,
        ops: &roc::RocOps,
        path: &RocStr,
    ) -> Result<RocList<u8>, RocSingleTagWrapper<IOErr>> {
        std::fs::read(path.as_str())
            .map(|s| RocList::from_slice(&s, ops))
            .map_err(|e| IOErr::from_io_error(&e, ops).into())
    }

    #[fallible]
    fn file_write_bytes(
        &mut self,
        ops: &roc::RocOps,
        path: &RocStr,
        bytes: &RocList<u8>,
    ) -> Result<(), RocSingleTagWrapper<IOErr>> {
        std::fs::write(path.as_str(), bytes.as_slice())
            .map_err(|e| IOErr::from_io_error(&e, ops).into())
    }

    #[fallible]
    fn file_read_utf8(
        &mut self,
        ops: &roc::RocOps,
        path: &RocStr,
    ) -> Result<RocStr, RocSingleTagWrapper<IOErr>> {
        std::fs::read_to_string(path.as_str())
            .map(|s| RocStr::from_str(s.as_str(), ops))
            .map_err(|e| IOErr::from_io_error(&e, ops).into())
    }

    #[fallible]
    fn file_write_utf8(
        &mut self,
        ops: &roc::RocOps,
        path: &RocStr,
        s: &RocStr,
    ) -> Result<(), RocSingleTagWrapper<IOErr>> {
        std::fs::write(path.as_str(), s.as_str()).map_err(|e| IOErr::from_io_error(&e, ops).into())
    }

    #[fallible]
    fn file_delete(
        &mut self,
        ops: &roc::RocOps,
        path: &RocStr,
    ) -> Result<(), RocSingleTagWrapper<IOErr>> {
        std::fs::remove_file(path.as_str()).map_err(|e| IOErr::from_io_error(&e, ops).into())
    }

    #[fallible]
    fn path_is_file(
        &mut self,
        ops: &roc::RocOps,
        path: &RocStr,
    ) -> Result<bool, RocSingleTagWrapper<IOErr>> {
        std::path::Path::new(path.as_str())
            .symlink_metadata()
            .map(|m| m.is_file())
            .map_err(|e| IOErr::from_io_error(&e, ops).into())
    }

    #[fallible]
    fn path_is_dir(
        &mut self,
        ops: &roc::RocOps,
        path: &RocStr,
    ) -> Result<bool, RocSingleTagWrapper<IOErr>> {
        std::path::Path::new(path.as_str())
            .symlink_metadata()
            .map(|m| m.is_dir())
            .map_err(|e| IOErr::from_io_error(&e, ops).into())
    }

    #[fallible]
    fn path_is_sym_link(
        &mut self,
        ops: &roc::RocOps,
        path: &RocStr,
    ) -> Result<bool, RocSingleTagWrapper<IOErr>> {
        std::path::Path::new(path.as_str())
            .symlink_metadata()
            .map(|m| m.is_symlink())
            .map_err(|e| IOErr::from_io_error(&e, ops).into())
    }

    #[fallible]
    fn random_seed_u64(&mut self, ops: &roc::RocOps) -> Result<u64, RocSingleTagWrapper<IOErr>> {
        Ok(roc_random::random_u64(ops)?)
    }

    #[fallible]
    fn random_seed_u32(&mut self, ops: &roc::RocOps) -> Result<u32, RocSingleTagWrapper<IOErr>> {
        Ok(roc_random::random_u32(ops)?)
    }

    fn sleep_millis(&mut self, _ops: &roc::RocOps, millis: &u64) {
        std::thread::sleep(std::time::Duration::from_millis(*millis));
    }

    fn stderr_line(&mut self, _ops: &roc::RocOps, message: &RocStr) {
        eprintln!("{}", message.as_str());
    }

    fn stderr_write(&mut self, _ops: &roc::RocOps, message: &RocStr) {
        eprint!("{}", message.as_str());
    }

    #[fallible]
    fn stdin_line(&mut self, ops: &roc::RocOps) -> Result<RocStr, RocSingleTagWrapper<IOErr>> {
        let mut buf = String::with_capacity(1024);
        stdin()
            .read_line(&mut buf)
            .map(|_| RocStr::from_str(buf.as_str().trim_end_matches('\n'), ops))
            .map_err(|e| IOErr::from_io_error(&e, ops).into())
    }

    #[fallible]
    fn stdin_read_to_end(
        &mut self,
        ops: &roc::RocOps,
    ) -> Result<RocList<u8>, RocSingleTagWrapper<IOErr>> {
        let mut buf = Vec::with_capacity(1024);
        stdin()
            .read_to_end(&mut buf)
            .map(|_| RocList::from_slice(&buf, ops))
            .map_err(|e| IOErr::from_io_error(&e, ops).into())
    }

    fn stdout_line(&mut self, _ops: &roc::RocOps, message: &RocStr) {
        println!("{}", message.as_str());
    }

    fn stdout_write(&mut self, _ops: &roc::RocOps, message: &RocStr) {
        print!("{}", message.as_str());
    }

    #[fallible]
    fn tty_enable_raw_mode(&mut self, ops: &roc::RocOps) -> Result<(), RocSingleTagWrapper<IOErr>> {
        crossterm::terminal::enable_raw_mode().map_err(|e| IOErr::from_io_error(&e, ops).into())
    }

    #[fallible]
    fn tty_disable_raw_mode(
        &mut self,
        ops: &roc::RocOps,
    ) -> Result<(), RocSingleTagWrapper<IOErr>> {
        crossterm::terminal::disable_raw_mode().map_err(|e| IOErr::from_io_error(&e, ops).into())
    }

    fn utc_now(&mut self, _ops: &roc::RocOps) -> u128 {
        let since_epoch = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time went backwards");
        since_epoch.as_nanos()
    }
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

    let host = RocHost::builder().build(Host);

    host.run(args)
}

platform_init!(init);
