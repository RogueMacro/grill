using System;
using System.Collections;

namespace Click;

class CLI
{
	public String ProgramName;
	public Version? Version = null;
	public String About = null;

	public static CLI Context { get; private set; }
	private String _error ~ delete _;

	public readonly Commands Commands = new .() ~ delete _;

	public this(String name)
	{
		ProgramName = name;
	}

	public void Run(StringView command, params Span<StringView> args)
	{
		Run(command, args);
	}

	public Result<void> Run(StringView command, Span<StringView> args)
	{
		Context = this;
		return Commands.Dispatch(command, args);
	}

	public void Report(String error)
	{
		delete _error;
		_error = error;
	}

	public void Help()
	{
		scope HelpCommand().Run();
	}
}