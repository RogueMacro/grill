using System;
using System.Collections;

namespace Click;

class CLI
{
	public String ProgramName;
	public Version? Version = null;
	public String About = null;

	public static CLI Context { get; private set; }

	public readonly Commands Commands = new .() ~ delete _;

	public this(String name)
	{
		ProgramName = name;

		Context = this;
	}

	public Result<void, CommandError> Run(StringView command, params Span<StringView> args)
	{
		return Run(command, args);
	}

	public Result<void, CommandError> Run(StringView command, Span<StringView> args)
	{
		//if (Commands.Dispatch(command, args) case .Err)
		//{
		//	if (!Commands.ParseError.IsEmpty)
		//}

		let result = Commands.Dispatch(command, args);
		let parseError = Commands.ParseError;

		switch ((result case .Ok, parseError.IsEmpty))
		{
		case (false, false):
			return .Err(.CommandFailed);
		case (false, true):
			return .Err(.ParseError(parseError));
		default:
			return .Ok;
		}
	}

	public void Help()
	{
		scope HelpCommand().Run();
	}

	public enum CommandError
	{
		case ParseError(String err);
		case CommandFailed;
	}
}