using System;
using System.Collections;
using System.IO;

using BuildTools.Git;
using Click;
using Serialize;
using Toml;

using Grill.Commands;
using Grill.Console;
using Grill.Resolver;

namespace Grill
{
    class Program
    {
		static CLI CLI = new .("grill") {
			Version = .(0, 1, 0),
			About = "A package manager for the Beef Programming Language"
		} ~ delete _;

        public static int Main(String[] args)
        {
			GConsole.CursorVisible = false;

			Git.git_libgit2_init();

			CLI.Commands.RegisterAll();

			Paths.ClearTemporary();

			//Repl();
			if (Run("make testproj") case .Err)
			{
				Console.Write($"\n{Styled("[error]")..Bright()..Red()} ");
				CLI.PrintErrorStackTrace();
			}

			Git.git_libgit2_shutdown();

			GConsole.CursorVisible = true;
			Console.Read().IgnoreError();
            return 0;
        }

		static Result<void> Run(Arguments arguments)
		{
			if (arguments.Subcommand() case .Ok(let sub))
				return CLI.Run(sub.cmd, sub.args);
			
			CLI.Help();
			return .Ok;
		}

		static Result<void> Run(StringView input)
		{
			let arguments = scope Arguments(input);
			return Run(arguments);
		}

		static void Repl()
		{
			Directory.CreateDirectory("repl");
			Directory.SetCurrentDirectory("repl");
			for (let dir in Directory.EnumerateDirectories("."))
				Directory.DelTree(dir.GetFileName(..scope .())..Insert(0, "./"));
			for (let file in Directory.EnumerateDirectories("."))
				File.Delete(file.GetFileName(..scope .()));

			while (true)
			{
				Console.Write("$ grill ");

				let input = Console.ReadLineEcho(.. scope .());
				if (input == "exit")
					break;

				if (Run(input) case .Err(let err))
				{
					Console.WriteLine($"{Styled("[Error] ")..Bright()..Red()} ");
					CLI.PrintErrorStackTrace();
				}
				Console.WriteLine();

				Console.Read();
				break;
			}
		}
    }
}
    