using System;

namespace Grill.Console;

static class GConsole
{
	static int32 offset;

	static CONSOLE_SCREEN_BUFFER_INFO GetCSBI()
	{
		let handle = Console.[Friend]GetStdHandle(Console.STD_OUTPUT_HANDLE);
		CONSOLE_SCREEN_BUFFER_INFO csbi;
		GetConsoleScreenBufferInfo(handle, out csbi);
		return csbi;
	}

	static void Check(bool newline)
	{
		let csbi = GetCSBI();
		if (csbi.mCursorPosition.y == csbi.mSize.y - 1 && newline)
			offset++;
	}

	public static int32 CursorLeft
	{
		get => Console.CursorLeft;
		set => Console.CursorLeft = value;
	}

	public static int32 CursorTop
	{
		get => Console.CursorTop + offset;
		set
		{
			int32 y = value - offset;
			let csbi = GetCSBI();
			if (y >= csbi.mSize.y)
			{
				Console.CursorTop = csbi.mSize.y - 1;
				WriteLine();
			}
			else
			{
				Console.CursorTop = y;
			}
		}
	}

	public static void Write(StringView line)
	{
		Check(false);
		Console.Write(line);
	}

	public static void Write(StringView fmt, params Object[] args)
	{
		Check(false);
		Console.Write(fmt, params args);
	}

	public static void Write(Object obj)
	{
		Check(false);
		Console.Write(obj);
	}

	public static void WriteLine()
	{
		Check(true);
		Console.WriteLine();
	}

	public static void WriteLine(StringView line)
	{
		Check(true);
		Console.WriteLine(line);
	}

	public static void WriteLine(StringView fmt, params Object[] args)
	{
		Check(true);
		Console.WriteLine(fmt, params args);
	}

	public static void WriteLine(Object obj)
	{
		Check(true);
		Console.WriteLine(obj);
	}

	[CRepr]
	struct SMALL_RECT {
	  public uint16 Left;
	  public uint16 Top;
	  public uint16 Right;
	  public uint16 Bottom;
	}

	[CRepr]
	struct CONSOLE_SCREEN_BUFFER_INFO
	{
		public COORD mSize;
		public COORD mCursorPosition;
		public uint16 mAttributes;
		public SMALL_RECT mWindow;
		public COORD mMaximumWindowSize;
	}

	[CRepr]
	struct COORD
	{
		public uint16 x;
		public uint16 y;
	}

	[CLink, CallingConvention(.Stdcall)]
	static extern Windows.IntBool GetConsoleScreenBufferInfo(Windows.Handle hConsoleOutput, out CONSOLE_SCREEN_BUFFER_INFO lpConsoleScreenBufferInfo);
}