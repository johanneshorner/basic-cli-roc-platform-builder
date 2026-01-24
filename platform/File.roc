File := [].{
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

	read_to_end! : Str => Try(Str, [FileErr(IOErr)])
}
