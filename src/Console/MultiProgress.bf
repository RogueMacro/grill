using System;
using System.Collections;
using System.Threading;
using System.Threading.Tasks;

namespace Grill.Console;

class MultiProgress
{
	int32 baseline;

	List<Progress> progressBars = new .() ~ DeleteContainerAndItems!(_);

	public this()
	{
		baseline = GConsole.CursorTop;
	}

	public void SetBaselineHere()
	{
		baseline = GConsole.CursorTop;
	}

	public void Add(Progress progress)
	{
		progress.[Friend]y = (.)(baseline + progressBars.Count);
		progressBars.Add(progress);
	}

	public void Remove(Progress progress)
	{
		progressBars.Remove(progress);
	}

	public void PushProgressY()
	{
		baseline++;
		for (let i in (0..<progressBars.Count).Reversed)
			progressBars[i].MoveDown();
	}

	public void Finish()
	{
		using (ConsoleLock.Acquire())
		{
			GConsole.CursorLeft = 0;
			GConsole.CursorTop = (.)(baseline + progressBars.Count);
		}
	}
}