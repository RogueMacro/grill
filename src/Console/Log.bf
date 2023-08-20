using System;

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

	public static void Info(StringView format, params Object[] args) => Print(Styled("[Info]")..Cyan(), format, params args);
	public static void Warning(StringView format, params Object[] args) => Print(Styled("[Warning]")..Yellow(), format, params args);
	public static void Error(StringView format, params Object[] args) => Print(Styled("[Error]")..Bright()..Red(), format, params args);

	public static void Print(Object prefix, StringView format, params Object[] args)
	{
		using (ConsoleLock.Acquire())
		{
			let origin = (GConsole.CursorLeft, GConsole.CursorTop);
			GConsole.CursorLeft = 0;
			GConsole.CursorTop = y;

			var prefix;
			String prefix = prefix.ToString(.. scope .());
			String pre = scope .();
			let spaces = 12 - GConsole.RealLength(prefix);
			if (spaces > 0)
				pre.Append(' ', spaces);
			pre.AppendF("{} ", prefix);


			String fmt = pre.ToString(.. scope .());
			fmt.Append(format);

			GConsole.WriteLine(fmt, params args);

			y++;
			(GConsole.CursorLeft, GConsole.CursorTop) = origin;
		}

		if (multi != null)
			multi.PushProgressY();
	}
}