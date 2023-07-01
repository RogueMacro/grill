using Click;

namespace System;

extension Result<T>
{
	public void Context(String message)
	{
		if (this case .Err)
			CLI.Context.Report(message);
	}

	public void Context(String format, params Object[] args)
	{
		if (this case .Err)
			CLI.Context.Report(format, params args);
	}
}