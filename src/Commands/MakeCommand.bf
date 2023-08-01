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

		Workspace workspace = scope .(Path);
		Try!(workspace.Open());

		return workspace.Make();
	}
}