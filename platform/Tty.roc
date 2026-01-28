## Provides functionality to change the behaviour of the terminal.
## This is useful for running an app like vim or a game in the terminal.
Tty := [].{
	## **NotFound** - An entity was not found, often a file.
	##
	## **PermissionDenied** - The operation lacked the necessary privileges to complete.
	##
	## **BrokenPipe** - The operation failed because a pipe was closed.
	##
	## **AlreadyExists** - An entity already exists, often a file.
	##
	## **Interrupted** - This operation was interrupted. Interrupted operations can typically be retried.
	##
	## **Unsupported** - This operation is unsupported on this platform. This means that the operation can never succeed.
	##
	## **OutOfMemory** - An operation could not be completed, because it failed to allocate enough memory.
	##
	## **Other** - A custom error that does not fall under any other I/O error kind.
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

	## Enable terminal [raw mode](https://en.wikipedia.org/wiki/Terminal_mode) to disable some default terminal bevahiour.
	##
	## This leads to the following changes:
	## - Input will not be echoed to the terminal screen.
	## - Input will be sent straight to the program instead of being buffered (= collected) until the Enter key is pressed.
	## - Special keys like Backspace and CTRL+C will not be processed by the terminal driver but will be passed to the program.
	enable_raw_mode! : () => [TtyErr(IOErr)]

	## Revert terminal to default behaviour
	disable_raw_mode! : () => [TtyErr(IOErr)]
}
