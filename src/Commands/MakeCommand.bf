using System;
using Click;

namespace Grill.Commands;

[Command("make", "Install the neccessary dependencies and make a workspace")]
class MakeCommand
{
	[Argument("path", "Path to the workspace", "p", ".")]
	public String Path ~ delete _;

	[Argument("quiet", "Don't print anything to the console", "q")]
	public bool Quiet;

	public Result<void> Run()
	{
		Workspace workspace = scope .(Path);
		Try!(workspace.Open());

		return workspace.Make(Quiet);
	}
}