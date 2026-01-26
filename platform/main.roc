platform ""
	requires {
		main! : List(Str) => Try({}, [Exit(U8), ..])
	}
	exposes [
		Cmd,
		Dir,
		Env,
		File,
		Http,
		Path,
		Random,
		Sleep,
		Stderr,
		Stdin,
		Stdout,
		Tty,
		Utc,
	]
	packages {}
	provides { main_for_host!: "main_for_host" }
	targets: {
		files: "targets/",
		exe: {
			x64musl: ["crt1.o", "libhost.a", app, "libc.a"],
			x64glibc: ["Scrt1.o", "crti.o", "libhost.a", app, "libc.so", "crtn.o", "libgcc_s.so.1"],
		},
	}

main_for_host! : List(Str) => U8
main_for_host! = |args| {
	result = main!(args)
	match result {
		Ok({}) => 0
		Err(Exit(code)) => code
		Err(other) => {
			Stderr.line!("Program exited with error: ${Str.inspect(other)}")
			1
		}
	}
}
