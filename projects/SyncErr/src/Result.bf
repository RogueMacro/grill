using SyncErr;

namespace System;

extension Result<T>
{
	public void Context(StringView message)
	{
		if (this case .Err)
			Errors.Report(message);
	}

	public void Context(StringView format, params Object[] args)
	{
		if (this case .Err)
			Errors.Report(format, params args);
	}

	public void Context(delegate void(String buffer) toString)
	{
		if (this case .Err)
		{
			String buffer = scope .();
			toString(buffer);
			Errors.Report(buffer);
		}
	}
}

extension Result<T, TErr>
{
	public void Context(String message)
	{
		if (this case .Err)
			Errors.Report(message);
	}

	public void Context(String format, params Object[] args)
	{
		if (this case .Err)
			Errors.Report(format, params args);
	}

	public void Context(delegate void(String buffer) toString)
	{
		if (this case .Err)
		{
			String buffer = scope .();
			toString(buffer);
			Errors.Report(buffer);
		}
	}
}