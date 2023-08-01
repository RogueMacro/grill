using System;
using System.Collections;

namespace SyncErr;

static class Errors
{
	public static List<String> ErrorStack = new .() ~ DeleteContainerAndItems!(_);

	public static void Report(StringView error)
	{
		ErrorStack.Add(new .(error));
	}

	public static void Report(StringView format, params Object[] args)
	{
		ErrorStack.Add(new String()..AppendF(format, params args));
	}

	public static void Clear()
	{
		ClearAndDeleteItems!(ErrorStack);
	}

	public static void PrintBacktrace()
	{
		if (ErrorStack.IsEmpty)
			return;

		Console.WriteLine(ErrorStack[^1]);
		for (let error in ErrorStack[..<^1].Reversed)
			Console.WriteLine($"\n    Caused by: {error}");
	}
}