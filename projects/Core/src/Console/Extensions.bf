namespace System;

extension Console
{
	public static Result<void> ReadLineEcho(String str)
	{
		while (true)
		{
			char8 c = Try!(Console.Read());

			if (c == '\n' || c == '\r')
			{
				Console.WriteLine();
				break;
			}

			if (c == '\b')
			{
				if (str.Length > 0)
				{
					str.RemoveFromEnd(1);

					Console.CursorLeft--;
					Console.Write(' ');
					Console.CursorLeft--;
				}
			}
			else
			{
				str.Append(c);
				Console.Write(c);
			}
		}

		return .Ok;
	}
}