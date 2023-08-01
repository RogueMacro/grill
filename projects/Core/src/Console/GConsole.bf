using System;

namespace Grill.Console;

static class GConsole
{
	public static bool Quiet = true;

	static int32 offset;

	static mixin Check(bool newline)
	{
		if (Quiet)
			return;

		let csbi = GetCSBI();
		if (csbi.mCursorPosition.y == csbi.mSize.y - 1 && newline)
			offset++;
	}

	public static int32 CursorLeft
	{
		get => Console.CursorLeft;
		set
		{
			if (!Quiet)
				Console.CursorLeft = value;
		}
	}

	public static int32 CursorTop
	{
		get => Console.CursorTop + offset;
		set
		{
			if (Quiet)
				return;

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

	public static bool CursorVisible
	{
		get => GetCursorInfo().visible;
		set
		{
			let handle = Console.[Friend]GetStdHandle(Console.STD_OUTPUT_HANDLE);
			var cursor = GetCursorInfo();
			cursor.visible = value;
			SetConsoleCursorInfo(handle, &cursor);
		}
	}

	public static void Write(StringView line)
	{
		Check!(false);
		Console.Write(line);
	}

	public static void Write(StringView fmt, params Object[] args)
	{
		Check!(false);
		Console.Write(fmt, params args);
	}

	public static void Write(Object obj)
	{
		Check!(false);
		Console.Write(obj);
	}

	public static void WriteLine()
	{
		Check!(true);
		Console.WriteLine();
	}

	public static void WriteLine(StringView line)
	{
		Check!(true);
		Console.WriteLine(line);
	}

	public static void WriteLine(StringView fmt, params Object[] args)
	{
		Check!(true);
		Console.WriteLine(fmt, params args);
	}

	public static void WriteLine(Object obj)
	{
		Check!(true);
		Console.WriteLine(obj);
	}

	public static int RealLength(StringView msg)
	{
		char32* buf = new char32[64]*;
		defer delete buf;
		System.Text.UTF32.Encode(msg, buf, 64);

		int len = 0;
		bool escape = false;
		for (int i = 0; i < 64; i++)
		{
			let c = buf[i];
			if (c == '\0')
				break;

			if (escape)
			{
				if (c == 'm')
					escape = false;
			}
			else if (c == '\x1B')
				escape = true;
			else
				len++;
		}

		return len;
	}

	static CONSOLE_SCREEN_BUFFER_INFO GetCSBI()
	{
		let handle = Console.[Friend]GetStdHandle(Console.STD_OUTPUT_HANDLE);
		CONSOLE_SCREEN_BUFFER_INFO csbi;
		GetConsoleScreenBufferInfo(handle, out csbi);
		return csbi;
	}

	static CONSOLE_CURSOR_INFO GetCursorInfo()
	{
		let handle = Console.[Friend]GetStdHandle(Console.STD_OUTPUT_HANDLE);
		CONSOLE_CURSOR_INFO cursor;
		GetConsoleCursorInfo(handle, out cursor);
		return cursor;
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

	[CRepr]
	struct CONSOLE_CURSOR_INFO
	{
		public uint32 size;
		public bool visible;
	}

	[CLink, CallingConvention(.Stdcall)]
	static extern Windows.IntBool GetConsoleScreenBufferInfo(Windows.Handle hConsoleOutput, out CONSOLE_SCREEN_BUFFER_INFO lpConsoleScreenBufferInfo);

	[CLink, CallingConvention(.Stdcall)]
	static extern Windows.IntBool GetConsoleCursorInfo(Windows.Handle hConsoleOutput, out CONSOLE_CURSOR_INFO lpConsoleCursorInfo);

	[CLink, CallingConvention(.Stdcall)]
	static extern Windows.IntBool SetConsoleCursorInfo(Windows.Handle hConsoleOutput, CONSOLE_CURSOR_INFO* lpConsoleCursorInfo);
}