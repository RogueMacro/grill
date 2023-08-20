using SyncErr;

namespace System;

extension Result<T>
{
	public void Context(StringView message)
	{
		if (this case .Err)
			Errors.Report(message);
		else
			Errors.Clear();
	}

	public void Context(StringView format, params Object[] args)
	{
		if (this case .Err)
			Errors.Report(format, params args);
		else
			Errors.Clear();
	}

	public void Context(delegate void(String buffer) toString)
	{
		if (this case .Err)
		{
			String buffer = scope .();
			toString(buffer);
			Errors.Report(buffer);
		}
		else
			Errors.Clear();
	}

	public Result<U> Map<U>(U value)
	{
		if (this case .Err)
			return .Err;
		return value;
	}

	public Result<U> MapLazy<U>(delegate U(T) map)
	{
		if (this case .Ok(let val))
			return map(val);
		return .Err;
	}
}

extension Result<T, TErr>
{
	public void Context(String message)
	{
		if (this case .Err)
			Errors.Report(message);
		else
			Errors.Clear();
	}

	public void Context(String format, params Object[] args)
	{
		if (this case .Err)
			Errors.Report(format, params args);
		else
			Errors.Clear();
	}

	public void Context(delegate void(String buffer) toString)
	{
		if (this case .Err)
		{
			String buffer = scope .();
			toString(buffer);
			Errors.Report(buffer);
		}
		else
			Errors.Clear();
	}

	public Result<T, UErr> MapErr<UErr>(delegate UErr(TErr) map)
	{
		if (this case .Err(let err))
			return .Err(map(err));
		return .Ok(this);
	}	

	public Result<T> Convert()
	{
		if (this case .Ok(let val))
			return val;
		return .Err;
	}
}