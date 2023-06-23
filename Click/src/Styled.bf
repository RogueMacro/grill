using System;
using System.Collections;

namespace Click
{
	static
	{
		public static StyledObject<T> Styled<T>(T val)
		{
			return StyledObject<T>(val);
		}
	}

	struct StyledObject<T>
	{
		private T Object;

		private Result<Color> Foreground = .Err;
		private Result<Color> Background = .Err;

		private bool FgBright = false;
		private bool BgBright = false;

		private ConsoleAttribute Attributes = .None;

		public this(T object)
		{
			Object = object;
		}

		public void Black() mut =>   Foreground = .Ok(.Black);
		public void Red() mut =>     Foreground = .Ok(.Red);
		public void Green() mut =>   Foreground = .Ok(.Green);
		public void Yellow() mut =>  Foreground = .Ok(.Yellow);
		public void Blue() mut =>    Foreground = .Ok(.Blue);
		public void Magenta() mut => Foreground = .Ok(.Magenta);
		public void Cyan() mut =>    Foreground = .Ok(.Cyan);
		public void White() mut =>   Foreground = .Ok(.White);
		public void Color256(uint8 x) mut => Foreground = .Ok(.Color256(x));

		public void OnBlack() mut =>   Background = .Ok(.Black);
		public void OnRed() mut =>     Background = .Ok(.Red);
		public void OnGreen() mut =>   Background = .Ok(.Green);
		public void OnYellow() mut =>  Background = .Ok(.Yellow);
		public void OnBlue() mut =>    Background = .Ok(.Blue);
		public void OnMagenta() mut => Background = .Ok(.Magenta);
		public void OnCyan() mut =>    Background = .Ok(.Cyan);
		public void OnWhite() mut =>   Background = .Ok(.White);
		public void OnColor256(uint8 x) mut => Background = .Ok(.Color256(x));

		public void Bright() mut => FgBright = true;
		public void OnBright() mut => BgBright = true;

		public void Bold() mut =>       Attributes |= .Bold;
		public void Dim() mut =>        Attributes |= .Dim;
		public void Italic() mut =>     Attributes |= .Italic;
		public void Underlined() mut => Attributes |= .Underlined;
		public void Blink() mut =>      Attributes |= .Blink;
		public void Reverse() mut =>    Attributes |= .Reverse;
		public void Hidden() mut =>     Attributes |= .Hidden;

		public override void ToString(String strBuffer)
		{
			bool reset = false;

			if (Foreground case .Ok(let fg))
			{
				if (fg case .Color256(let x))
					strBuffer.AppendF("\x1b[38;5;{}m", x);
				else if (FgBright)
					strBuffer.AppendF("\x1b[38;5;{}m", fg.AnsiCode() + 8);
				else
					strBuffer.AppendF("\x1b[{}m", fg.AnsiCode() + 30);

				reset = true;
			}

			if (Background case .Ok(let bg))
			{
				if (bg case .Color256(let x))
					strBuffer.AppendF("\x1b[48;5;{}m", x);
				else if (FgBright)
					strBuffer.AppendF("\x1b[48;5;{}m", bg.AnsiCode() + 8);
				else
					strBuffer.AppendF("\x1b[{}m", bg.AnsiCode() + 40);

				reset = true;
			}

			for (uint8 i = 0; i < 7; i++)
			{
				let attribute = Attributes & (.)(1 << i);
				if (attribute != .None)
				{
					strBuffer.AppendF("\x1b[{}m", attribute.AnsiCode());
					reset = true;
				}
			}

			
			Object.ToString(strBuffer);

			if (reset)
				strBuffer.Append("\x1b[0m");
		}
	}

	enum ConsoleAttribute
	{
		None       = 0,
		Bold       = 1 << 0,
		Dim        = 1 << 1,
		Italic     = 1 << 2,
		Underlined = 1 << 3,
		Blink      = 1 << 4,
		Reverse    = 1 << 5,
		Hidden     = 1 << 6
	}

	extension ConsoleAttribute
	{
		public bool Has(Self attribute) => this & attribute != 0;

		public uint8 AnsiCode()
		{
			switch (this)
			{
			case None:       return 0;
			case Bold:       return 1;
			case Dim:        return 2;
			case Italic:     return 3;
			case Underlined: return 4;
			case Blink:      return 5;
			case Reverse:    return 7;
			case Hidden:     return 8;
			}
		}
	}

	enum Color
	{
		case Black;
		case Red;
		case Green;
		case Yellow;
		case Blue;
		case Magenta;
		case Cyan;
		case White;
		case Color256(uint8);

		public uint8 AnsiCode()
		{
			switch (this)
			{
			case .Black:   return 0;
			case .Red:     return 1;
			case .Green:   return 2;
			case .Yellow:  return 3;
			case .Blue:    return 4;
			case .Magenta: return 5;
			case .Cyan:    return 6;
			case .White:   return 7;
			case .Color256(let x): return x;
			}
		}
	}
}