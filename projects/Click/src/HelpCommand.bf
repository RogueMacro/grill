using System;
using System.Collections;
using SyncErr;
using Click.Internal;

namespace Click;

[Command("help", "Print this message or the help of the given subcommand(s)")]
class HelpCommand
{
	[Argument]
	public String Command ~ delete _;

	public Result<void> Run()
	{
		let ctx = CLI.Context;

		if (Command == null)
		{
			Console.Write(Styled(ctx.ProgramName)..Green());
			if (ctx.Version != null)
				Console.Write($" {ctx.Version->Major}.{ctx.Version->Minor}.{ctx.Version->Build}");
			Console.WriteLine();

			if (ctx.About != null)
				Console.WriteLine(ctx.About);

			Console.WriteLine($"\n{Styled("USAGE:")..Yellow()}");
			Console.WriteLine($"    {ctx.ProgramName} [OPTIONS] [SUBCOMMAND]");

			int longestCommandName = 0;
			for (let name in ctx.Commands.Keys)
			{
				if (name.Length > longestCommandName)
					longestCommandName = name.Length;
			}

			Console.WriteLine($"\n{Styled("SUBCOMMANDS:")..Yellow()}");
			for (let (command, type) in ctx.Commands)
			{
				Console.Write($"    {Styled(command)..Green()}");
				if (type.GetCustomAttribute<CommandAttribute>() case .Ok(let attr))
					Console.Write($"{scope String(' ', longestCommandName - command.Length + 4)}{attr.About}");
				Console.WriteLine();
			}
		}
		else if (ctx.Commands.ContainsKey(Command))
		{
			let type = ctx.Commands[Command];
			let attr = Try!(type.GetCustomAttribute<CommandAttribute>());

			Console.WriteLine(Styled(scope $"{ctx.ProgramName}-{Command}")..Green());
			if (attr.About != null)
				Console.WriteLine(attr.About);

			Console.WriteLine($"\n{Styled("USAGE:")..Yellow()}");
			Console.WriteLine($"    {ctx.ProgramName} {attr.Name} [OPTIONS]");

			List<ArgAttribute> args = scope .();
			int longestArg = 0;
			bool anyShort = false;
			for (let field in type.GetFields())
			{
				if (field.GetCustomAttribute<ArgAttribute>() case .Ok(var arg))
				{
					if (arg.Name == null)
						arg.Name = scope:: .(field.Name);

					if (arg.Name.Length > longestArg)
						longestArg = arg.Name.Length;

					if (arg.Short != null)
						anyShort = true;

					args.Add(arg);
				}
			}

			Console.WriteLine($"\n{Styled("OPTIONS:")..Yellow()}");
			for (let arg in args)
			{
				Console.Write("    ");
				if (arg.Short != null)
					Console.Write($"{Styled(scope $"-{arg.Short}")..Green()}, ");
				else if (anyShort)
					Console.Write("    ");
				Console.Write($"{Styled(scope $"--{arg.Name}")..Green()}");

				Console.Write($"{scope String(' ', longestArg - arg.Name.Length + 4)}{(arg.About ?? "")} {(arg.Default != null ? scope $"[default: {arg.Default}]" : "")}");
				Console.WriteLine();
			}
		}
		else
		{
			Errors.Report($"Unknown command '{Command}'");
			return .Err;
		}

		return .Ok;
	}

	private void DisplayHelp(Type command)
	{
		let cli = CLI.Context;

		Console.WriteLine(Styled(scope $"{cli.ProgramName}-{Command}")..Green());
		let type = cli.Commands[Command];
		if (type.GetCustomAttribute<CommandAttribute>() case .Ok(let attr) && attr.About != null)
			Console.WriteLine(attr.About);
	}
}