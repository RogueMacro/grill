using System;

namespace SyncErr;

public static
{
	public static mixin Bail(StringView message)
	{
		return (.Err(?))..Context(message);
	}
}