using System;
using System.IO;
using Click;

namespace Grill.Commands;

[Command("new", "Create a new workspace and project")]
class NewCommand
{
	[Argument(Required=true)]
	public String Path ~ delete _;

	public Result<void> Run()
	{
		if (Path == null)
		{
			CLI.Context.Report(new $"Path not supplied");
			return .Err;
		}

		Directory.CreateDirectory(Path);
		return .Ok;
	}
}