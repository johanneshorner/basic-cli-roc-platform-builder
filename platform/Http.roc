Http := [].{
	RequestErr := [
		Builder,
		Redirect,
		Status(U16),
		Timeout,
		Request,
		Body,
		Decode,
		Upgrade,
		InvalidMethod,
		InvalidHeaderName,
		InvalidHeaderValue,
		Other,
	]

	Method := [
		Options,
		Get,
		Post,
		Put,
		Delete,
		Head,
		Trace,
		Connect,
		Patch,
		Extension(Str),
	]

	Header : { name : Str, value : Str }

	Request : {
		method : Method,
		headers : List(Header),
		uri : Str,
		body : List(U8),
	}

	Response : {
		status : U16,
		headers : List(Header),
		body : List(U8),
	}

	send! : Request => Try(Response, [HttpErr(RequestErr)])
}
