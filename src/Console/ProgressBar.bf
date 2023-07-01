using System;

namespace Grill.Console;

class ProgressBar : Progress
{
	int current = -1;
	int progressLength => (.)Math.Floor((float)current / steps * length);

	int steps;
	int length;
	int offsetX;

	public readonly String Text = new .() ~ delete _;

	int lastTextLength;

	public this(int steps, int length = 20, int offsetX = 13)
	{
		this.steps = steps;
		this.length = length;
		this.offsetX = offsetX;
	}

	public void UpdateText(StringView text)
	{
		Text.Set(text);
		RenderAtY();
	}

	public void Tick()
	{
		let prevLength = progressLength;
		current++;
		if (progressLength > prevLength)
			RenderAtY();
	}

	void RenderAtY()
	{
		using (ConsoleLock.Acquire())
		{
			GConsole.CursorLeft = 0;
			GConsole.CursorTop = y;
			Render();
		}
	}

	public override void ClearLine()
	{
		let n = offsetX + length + 2 + lastTextLength;
		for (let _ in 0..<n)
			GConsole.Write(' ');
	}

	public override void Render()
	{
		for (let _ in 0..<offsetX)
			GConsole.Write(' ');

		GConsole.Write('[');

		if (progressLength > 0)
		{
			for (let _ in 0..<progressLength)
				GConsole.Write('=');
		}

		if (progressLength < length)
		{
			GConsole.Write('>');
			for (let _ in progressLength..<length-1)
				GConsole.Write(' ');
		}

		GConsole.Write(']');

		GConsole.Write(" {}", Text);

		if (lastTextLength > Text.Length)
		{
			for (let _ in Text.Length...lastTextLength)
				GConsole.Write(' ');
		}

		lastTextLength = Text.Length;
	}

	public void Finish()
	{
		using (ConsoleLock.Acquire())
		{
			GConsole.CursorLeft = 0;
			GConsole.CursorTop = y;
			ClearLine();
		}
	}
}