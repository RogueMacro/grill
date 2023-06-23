using System;

namespace Click;

[Command("help", "Print this message or the help of the given subcommand(s)")]
class HelpCommand
{
	[Argument]
	public String Command ~ delete _;

	public Result<void> Run()
	{
		let cli = CLI.Context;

		if (Command == null)
		{
			Console.Write(Styled(cli.ProgramName)..Green());
			if (cli.Version != null)
				Console.Write($" {cli.Version->Major}.{cli.Version->Minor}.{cli.Version->Build}");
			Console.WriteLine();

			if (cli.About != null)
				Console.WriteLine(cli.About);

			Console.WriteLine($"\n{Styled("USAGE:")..Yellow()}");
			Console.WriteLine($"    {cli.ProgramName} [OPTIONS] [SUBCOMMAND]");

			int longestCommandName = 0;
			for (let name in cli.Commands.Keys)
			{
				if (name.Length > longestCommandName)
					longestCommandName = name.Length;
			}

			Console.WriteLine($"\n{Styled("SUBCOMMANDS:")..Yellow()}");
			for (let (command, type) in cli.Commands)
			{
				Console.Write($"    {Styled(command)..Green()}");
				if (type.GetCustomAttribute<CommandAttribute>() case .Ok(let attr))
					Console.Write($"{scope String(' ', longestCommandName - command.Length + 4)}{attr.About}");
				Console.WriteLine();
			}
		}
		else
		{
			if (!cli.Commands.ContainsKey(Command))
			{
				cli.Report(new $"Unknown command '{Command}'");
				return .Err;
			}

			
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