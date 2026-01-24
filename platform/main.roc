platform ""
	requires {
		main! : List(Str) => Try({}, [Exit(U8), ..])
	}
	exposes [Cmd, Dir, Env, File, Stdout, Stdin, Stderr]
	packages {}
	provides { main_for_host!: "main_for_host" }
	targets: {
		files: "targets/",
		exe: {
			x64musl: ["crt1.o", "libhost.a", app, "libc.a"],
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
