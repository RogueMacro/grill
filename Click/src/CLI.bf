using System;
using System.Collections;

namespace Click;

class CLI
{
	public String ProgramName;
	public Version? Version = null;
	public String About = null;

	public static CLI Context { get; private set; }
	public String Error { get; private set; } ~ delete _;

	public readonly Commands Commands = new .() ~ delete _;

	public this(String name)
	{
		ProgramName = name;

		Context = this;
	}

	public void Run(StringView command, params Span<StringView> args)
	{
		Run(command, args);
	}

	public Result<void> Run(StringView command, Span<StringView> args)
	{
		return Commands.Dispatch(command, args);
	}

	public void Report(String error)
	{
		delete Error;
		Error = new .(error);
	}

	public void Report(String format, params Object[] args)
	{
		delete Error;
		Error = new .()..AppendF(format, params args);
	}

	public void Help()
	{
		scope HelpCommand().Run();
	}
}