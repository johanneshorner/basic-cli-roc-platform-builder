app [main!] { pf: platform "../platform/main.roc" }

import pf.File
import pf.Stdout
import pf.Stderr
import pf.Stdin
import pf.Cmd
import pf.Dir
import pf.Http

main! : List(Str) => Try({}, [Exit(U8), ..])
main! = |args| {
	file_path = match args {
		[_, path] => path
		_ => {
			Stdout.line!("Give me a path to a text file")
			return Err(Exit(25))
		}
	}

	File.write_utf8!(file_path, "helloa").map_err(|e| Write(e))?
	content = File.read_utf8!(file_path).map_err(|e| Read(e))?
	Stderr.line!("`File`: write(args[1]) -> read -> content: ${content}")

	stdin = Stdin.read_to_end!()->Str.from_utf8_lossy()
	Stdout.line!("`Stdin`: read_to_end -> in: ${stdin}")

	Cmd.new("ls").args(["-l", "-a"]).exec_cmd!().map_err(|e| Cmd(e))?
	Stderr.line!("`Cmd`: ls -la -> output: ${Str.inspect(Dir.list!("/home/johannes"))}")

	response = Http.send!({ method: Get, headers: [], uri: "https://google.com", body: [] }).map_err(|e| Get(e))?
	Stdout.line!("`Http`: send -> body: ${Str.from_utf8_lossy(response.body)}")

	Ok({})
}
