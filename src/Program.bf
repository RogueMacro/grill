using System;
using System.Collections;
using System.IO;

using BuildTools.Git;
using Click;
using Serialize;
using Toml;
using SyncErr;
using Iterators;

using Grill.Beef;
using Grill.Console;
using Grill.CLI.Commands;
using Grill.Resources;
using System.Diagnostics;

namespace Grill.CLI
{
    class Program
    {
		static CLI CLI = new .("grill") {
			Version = .(0, 1, 0),
			About = "A package manager for the Beef Programming Language"
		} ~ delete _;

        public static int Main(String[] args)
        {
			Git.git_libgit2_init();
			ResourceManager.Init<ResourceProvider>();
			Paths.ClearTemporary();
			CLI.Commands.RegisterAll();

			Arguments arguments = scope .();
			for (let a in args)
				arguments.Add(StringView(a));
			if (Run(arguments) case .Err)
			{
				Console.Write($"\n{Styled("[Error]")..Bright()..Red()} ");
				Errors.PrintBacktrace();
			}

			Git.git_libgit2_shutdown();
            return 0;
        }

		static Result<void, CLI.CommandError> Run(Arguments arguments)
		{
			if (arguments.Subcommand() case .Ok(let sub))
				return CLI.Run(sub.cmd, sub.args);
			
			CLI.Help();
			return .Ok;
		}

		static Result<void, CLI.CommandError> Run(StringView input)
		{
			let arguments = scope Arguments(input);
			return Run(arguments);
		}

		static void Repl(bool replEnv = true)
		{
			if (replEnv)
			{
				Directory.CreateDirectory("repl");
				Directory.SetCurrentDirectory("repl");
				for (let dir in Directory.EnumerateDirectories("."))
					Directory.DelTree(dir.GetFileName(..scope .())..Insert(0, "./"));
				for (let file in Directory.EnumerateDirectories("."))
					File.Delete(file.GetFileName(..scope .()));
			}

			while (true)
			{
				Console.Write("$ grill ");

				let input = Console.ReadLineEcho(.. scope .());
				if (input == "exit")
					break;

				if (Run(input) case .Err(let err))
				{
					Console.Write(Styled("[Error] ")..Bright()..Red());
					Errors.PrintBacktrace();
				}

				Console.WriteLine();
			}
		}
    }
}
    