using System;
using System.IO;

using Grill.Beef;
using Grill.Resources;

using Click;
using Iterators;
using SyncErr;

namespace Grill.CLI.Commands;

[Command("new", "Create a new package")]
class NewCommand
{
	[Argument("path", "Where to place the package", "p", ".", true)]
	public String ProjectPath ~ delete _;

	[Argument("bin", "Create a console application")]
	public bool Binary;

	[Argument("lib", "Create a library")]
	public bool Library;

	[Argument("gui", "Create a GUI application")]
	public bool GUI;

	[Argument("test", "Create an integration test")]
	public bool Test;

	[Argument("name", "Name of the package")]
	public String Name ~ delete _;

	public Result<void> Run()
	{
		TargetType targetType;
		if (Binary)
			targetType = .Binary;
		else if (Library)
			targetType = .Library;
		else if (GUI)
			targetType = .GUI;
		else if (Test)
			targetType = .Test;
		else
			targetType = .Binary;

		if (targetType case .Test)
		{
			if (Name == null)
				Bail!("Name has to be specified for integration test");

			Package package = scope .();
			Try!(package.Open(ProjectPath));
			return package.CreateIntegrationTest(Name);
		}	

		Package package = scope .();
		return package.Create(ProjectPath, Name);
	}
}