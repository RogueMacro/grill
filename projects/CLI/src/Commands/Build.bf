using System;
using Click;
using Grill.Beef;

namespace Grill.CLI.Commands;

[Command("build", "Build the workspace")]
class Build
{
	public Result<void> Run()
	{
		let exitCode = BeefBuild.Build();

		Console.WriteLine(exitCode);

		return .Ok;
	}
}