using System;
using Grill.Console;
using Click;

namespace Grill.CLI.Commands;

[Command("make", "Install the neccessary dependencies and make a workspace")]
class MakeCommand
{
	[Argument("path", "Path to the workspace", "p", ".")]
	public String Path ~ delete _;

	[Argument("quiet", "Don't print anything to the console", "q")]
	public bool Quiet;

	public Result<void> Run()
	{
		GConsole.Quiet = Quiet;
		GConsole.CursorVisible = false;
		defer { GConsole.CursorVisible = true; }

		Package package = scope .();
		Try!(package.Open(Path));
		return package.Make();
	}
}