using System;
using System.Collections;
using System.Diagnostics;
using System.IO;
using Click;
using Iterators;
using SyncErr;
using Grill.Beef;
using Grill.Console;

namespace Grill.CLI.Commands;

[Command("test", "Run tests")]
class TestCommand
{
	[Argument("unittests", "Run unittests")]
	public bool RunUnitTests;

	[Argument("integration", "Run integration tests")]
	public bool RunIntegrationTests;

	[Argument("make", "Make the test projects")]
	public bool Make;

	[Argument("test", "Run a specific test")]
	public String Test ~ delete _;

	public Result<void> Run()
	{
		if (RunUnitTests == false && RunIntegrationTests == false && Test == null)
			RunUnitTests = RunIntegrationTests = true;

		if (Make)
		{
			let tests = GetTests!();
			Console.WriteLine($"\n- Making {tests.Count} tests");
			for (let test in tests)
			{
				let name = Path.GetFileName(test, .. scope .());
				Console.Write($"make {name} ... ");
				if (MakeTest(test) case .Err)
				{
					Console.WriteLine();
					return .Err;
				}

				Console.WriteLine(Styled("OK")..Bright()..Green());
			}

			return .Ok;
		}

		if (RunUnitTests)
		{
			Console.WriteLine("Running unittests");

			if (BeefBuild.Test() != 0)
				Bail!("Unittests failed");
		}

		if (RunIntegrationTests)
		{
			int ok = 0;
			int failed = 0;

			let tests = GetTests!();
			Console.WriteLine($"\n- Running {tests.Count} tests -");

			Stopwatch stopwatch = scope .(true);
			for (let test in tests)
			{
				let code = Try!(IntegrationTest(test));
				if (code == 0)
					ok++;
				else
					failed++;
			}

			stopwatch.Stop();
			float secs = stopwatch.ElapsedMilliseconds / 1000f;
			Console.WriteLine($"\nResults: {ok} passed; {failed} failed; in {secs:F2}s");
		}

		if (Test != null)
		{
			let path = Path.InternalCombine(.. scope .(), "tests", Test);
			if (!Directory.Exists(path))
				Bail!(scope $"{Test} not found in tests/");

			Try!(IntegrationTest(path));
		}

		return .Ok;
	}

	Result<int> IntegrationTest(StringView path)
	{
		let name = Path.GetFileName(path, .. scope .());
		Console.Write($"test {name} ... ");

		Try!(MakeTest(path));

		int exit = BeefBuild.Start(.() {
			Run = true,
			Workspace = path
		}, showOutput:true);

		if (exit == 0)
			Console.WriteLine(Styled("OK")..Bright()..Green());
		else
			Console.WriteLine(Styled("FAILED")..Bright()..Red());

		return .Ok(exit);
	}

	Result<void> MakeTest(StringView path)
	{
		Package package = scope .();
		Try!(package.Open(path, "."));
		return package.Make();
	}

	mixin GetTests()
	{
		List<String> list = scope:mixin .();
		for (let dir in Directory.EnumerateDirectories("tests"))
		{
			let path = dir.GetFilePath(.. scope:mixin .());
			if (Path.GetFileName(path, .. scope .()) == "common")
				continue;
			list.Add(path);
		}
		list
	}
}