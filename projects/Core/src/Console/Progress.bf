namespace Grill.Console;

abstract class Progress
{
	protected bool isFinished;

	protected int32 y;

	public void MoveDown()
	{
		using (ConsoleLock.Acquire())
		{
			let origin = (GConsole.CursorLeft, GConsole.CursorTop);

			GConsole.CursorLeft = 0;
			GConsole.CursorTop = y;

			ClearLine();
			GConsole.WriteLine();
			Render();

			y++;

			OnMoveDown();

			(GConsole.CursorLeft, GConsole.CursorTop) = origin;
		}
	}

	public abstract void ClearLine();
	public abstract void Render();

	protected virtual void OnMoveDown() {}
}