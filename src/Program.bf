using System;
using System.Collections;
using System.IO;

using BuildTools.Git;
using Click;
using Serialize;
using Toml;

using Grill.Commands;

namespace Grill
{
	struct AttrAttribute : AttrAttribute<0>
	{

	}

	struct AttrAttribute<S> : Attribute
		where S : const int
	{
		public String[S] Subcommands;
	}

	[Attr<2>(Subcommands=.("", ""))]
    class Program
    {
		static CLI CLI = new .("grill") {
			Version = .(0, 1, 0),
			About = "A package manager for the Beef Programming Language"
		} ~ delete _;

        public static int Main(String[] args)
        {
			Git.git_libgit2_init();

			CLI.Commands.RegisterAll();

#if DEBUG
			Directory.CreateDirectory("repl");
			Directory.SetCurrentDirectory("repl");
			for (let dir in Directory.EnumerateDirectories("."))
				Directory.DelTree(dir.GetFileName(..scope .())..Insert(0, "./"));
			for (let file in Directory.EnumerateDirectories("."))
				File.Delete(file.GetFileName(..scope .()));
			
			Repl();
			//Console.WriteLine("> grill help");
			//CLI.Help();
#else
			Run(args);
#endif

			Git.git_libgit2_shutdown();
			//Console.WriteLine("\n\n\n---");
			Console.Read().IgnoreError();
            return 0;
        }

		static Result<void> Run(StringView input)
		{
			let arguments = scope Arguments(input);
			if (arguments.Subcommand() case .Ok(let sub))
				return CLI.Run(sub.cmd, sub.args);
			
			CLI.Help();
			return .Ok;
		}

		static void Repl()
		{
			while (true)
			{
				Console.Write("$ grill ");

				let input = Console.ReadLine(.. scope .());
				Console.WriteLine();
				//let input = "help";
				if (Run(input) case .Err(let err))
					Console.WriteLine($"{Styled("[Error] ")..Red()} {CLI.[Friend]_error}");
				Console.WriteLine();
				break;
			}
		}
    }
}
    