Stdin := [].{
	IOErr := [
		NotFound,
		PermissionDenied,
		BrokenPipe,
		AlreadyExists,
		Interrupted,
		Unsupported,
		OutOfMemory,
		Other(Str),
	]

	line! : () => Try(Str, [StdinErr(IOErr)])
	read_to_end! : () => Try(List(U8), [StdinErr(IOErr)])
}
