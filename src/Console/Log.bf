using System;
using Click;

namespace Grill.Console;

static class Log
{
	static int32 y;
	static MultiProgress multi;

	public static void SetProgress(MultiProgress multiProgress)
	{
		multi = multiProgress;
	}

	public static void SetPosHere()
	{
		y = GConsole.CursorTop;
	}

	public static void Print(StringView prefix, Color color, StringView format, params Object[] args)
	{
		using (ConsoleLock.Acquire())
		{
			let origin = (GConsole.CursorLeft, GConsole.CursorTop);
			GConsole.CursorLeft = 0;
			GConsole.CursorTop = y;

			String pre = scope .();
			pre.AppendF("{0,12} ", prefix);
			var styledPre = Styled(pre);
			styledPre.[Friend]Foreground = color;
			styledPre.Bright();

			String fmt = styledPre.ToString(.. scope .());
			fmt.Append(format);

			GConsole.WriteLine(fmt, params args);

			y++;
			(GConsole.CursorLeft, GConsole.CursorTop) = origin;
		}

		if (multi != null)
			multi.PushProgressY();
	}
}