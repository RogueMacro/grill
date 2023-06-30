using System;
using System.IO;
using Click;

namespace Grill.Commands;

[Command("new", "Create a new workspace and project")]
class NewCommand
{
	[Argument("path", "Path to create the new workspace", "p", ".", true)]
	public String Path ~ delete _;

	public Result<void> Run()
	{
		Directory.CreateDirectory(Path);
		return .Ok;
	}
}