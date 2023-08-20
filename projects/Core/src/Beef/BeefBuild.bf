using System;
using System.Diagnostics;
using System.IO;

namespace Grill.Beef;

[StaticInitAfter(typeof(Paths))]
static class BeefBuild
{
	public static int Run()
	{
		return Start(.() {
			Run = true
		});
	}

	public static int Test()
	{
		return Start(.() {
		   	Test = true
		});
	}

	public static int Build(StringView path = ".")
	{
		if (path == ".")
			return Start(.());
		return Start(.() {
		   Workspace = path,
		});
	}

	public struct Args
	{
		/// Cleans the build cache when building
		public bool? Clean;

		/// Sets the config
		public StringView? Config;

		/// Generates startup code for an empty project
		public bool? Generate;

		/// Creates a new workspace and project
		public bool? New;

		/// Sets the platform
		public StringView? Platform;

		/// Compile and run the startup project in the workspace
		public bool? Run;

		/// Run tests
		public bool Test;

		/// Set verbosity level to: quiet/minimal/normal/detailed/diagnostic
		public Verbosity? Verbosity;

		/// Sets workspace path
		public StringView? Workspace;

		public enum Verbosity
		{
			quiet,
			minimal,
			normal,
			detailed,
			diagnostic
		}
	}

	static String exePath = Path.InternalCombine(.. new .(), Paths.BeefPath, "bin", "BeefBuild.exe") ~ delete _;

	public static int Start(Args args, bool showOutput = false)
	{
		String argStr = scope .("");
		if (args.Clean != null) argStr.Append(" -clean");
		if (args.Config != null) argStr.AppendF($" -config={args.Config}");
		if (args.Generate != null) argStr.Append(" -generate");
		if (args.New != null) argStr.Append(" -new");
		if (args.Platform != null) argStr.AppendF($" -platform={args.Platform}");
		if (args.Run != null) argStr.Append(" -run");
		if (args.Test) argStr.Append(" -test");
		if (args.Verbosity != null) argStr.AppendF($" -verbosity={args.Verbosity}");
		if (args.Workspace != null) argStr.AppendF($" -workspace=\"{args.Workspace}\"");

		ProcessStartInfo startInfo = scope .();
		startInfo.SetFileName(exePath);
		startInfo.SetArguments(argStr);
		startInfo.UseShellExecute = false;
		startInfo.CreateNoWindow = !showOutput;
		//startInfo.RedirectStandardOutput = true;

		SpawnedProcess process = scope .();
		process.Start(startInfo);
		process.WaitFor();

		

		return process.ExitCode;
	}
}