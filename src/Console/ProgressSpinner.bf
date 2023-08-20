using System;
using System.Threading;
using System.Threading.Tasks;

namespace Grill.Console;

class ProgressSpinner : Progress
{
	String message ~ delete _;
	String finish ~ delete _;

	ThreadSpinner spinner ~ delete _;
	Task task ~ delete _;

	public this(StringView message, StringView finish, int y = -1)
	{
		this.message = new .(message);
		this.finish = new .(finish);
		this.y = (.)(y < 0 ? GConsole.CursorTop : y);
	}

	public void EnableSteadyTick(int interval)
	{
		spinner = new .(GConsole.RealLength(message) + 1, y, interval, message, finish);

		using (ConsoleLock.Acquire())
		{
			let origin = (GConsole.CursorLeft, GConsole.CursorTop);
			GConsole.CursorLeft = 0;
			GConsole.CursorTop = y;

			GConsole.Write(message);
			(GConsole.CursorLeft, GConsole.CursorTop) = origin;
		}

		task = new .(new () => {
			spinner.Spin();
		});

		task.Start();
	}

	public override void ClearLine()
	{
		if (isFinished)
		{
			let msgLength = GConsole.RealLength(finish) + GConsole.RealLength(StringView((char8*)&spinner.[Friend]end));
			for (int _ in 0..<msgLength)
				GConsole.Write(' ');
		}
		else
		{
			let msgLength = GConsole.RealLength(message) + 2;
			for (int _ in 0..<msgLength)
				GConsole.Write(' ');
		}
	}

	public override void Render()
	{
		if (isFinished)
		{
			GConsole.Write(finish);
			GConsole.WriteLine(spinner.[Friend]end);
		}
		else
		{
			GConsole.Write(message);
		}
	}

	protected override void OnMoveDown()
	{
		spinner.MoveDown();
	}

	public void Finish()
	{
		Interlocked.Store(ref spinner.Spin, false);
		task.Wait();
		isFinished = true;

		using (ConsoleLock.Acquire())
		{
			let origin = (GConsole.CursorLeft, GConsole.CursorTop);
			let msgLength = GConsole.RealLength(message);
			let finishLength = GConsole.RealLength(finish);

			GConsole.CursorLeft = 0;
			GConsole.CursorTop = y;

			GConsole.Write(finish);
			if (msgLength > finishLength)
				for (int _ in 0..<(msgLength - finishLength))
					GConsole.Write(' ');

			GConsole.WriteLine(spinner.[Friend]end);

			(GConsole.CursorLeft, GConsole.CursorTop) = origin;
		}
	}

	class ThreadSpinner
	{
		int32 x;
		int32 y;

		String characters = "|/-|/-\\";
		char16 end = '✔';
		int current;
		int32 interval;

		public bool Spin = true;

		public this(int x, int y, int interval, String message, String finish)
		{
			this.x = (.)x;
			this.y = (.)y;
			this.interval = (.)interval;
		}

		public void MoveDown()
		{
			int32 newY = Interlocked.Load(ref y);
			newY++;
			Interlocked.Store(ref y, newY);
		}

		public void Spin()
		{
			while (Interlocked.Load(ref Spin))
			{
				using (ConsoleLock.Acquire())
				{
					let pos = (GConsole.CursorLeft, GConsole.CursorTop);
					GConsole.CursorLeft = x;
					GConsole.CursorTop = y;
					if (current >= characters.Length)
						current = 0;
					let c = characters[current++];
					GConsole.Write(c);
					GConsole.CursorLeft = pos.0;
					GConsole.CursorTop = pos.1;
				}
				
				Thread.Sleep(interval);
			}

			using (ConsoleLock.Acquire())
			{
				let origin = (GConsole.CursorLeft, GConsole.CursorTop);
				GConsole.CursorLeft = x;
				GConsole.CursorTop = y;
				GConsole.Write(end);
				(GConsole.CursorLeft, GConsole.CursorTop) = origin;
			}
		}
	}
}