using System;
using Click;
using Grill.Beef;

namespace Grill.CLI.Commands;

[Command("run", "Run the startup project")]
class RunCommand
{
	public Result<void> Run()
	{
		if (BeefBuild.Run() == 0)
			return .Ok;
		return .Err;
	}
}