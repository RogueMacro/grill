using System;
using System.Threading;

namespace Grill.Console;

struct ConsoleLock : IDisposable
{
	static bool isLocked = false;

	private this()
	{
		bool acquired = false;
		while (!acquired)
			acquired = Interlocked.CompareStore(ref isLocked, false, true);
	}

	public static ConsoleLock Acquire()
	{
		return ConsoleLock();
	}

	public void Dispose()
	{
		Interlocked.Store(ref isLocked, false);
	}
}