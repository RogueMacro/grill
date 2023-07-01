using System;
using System.Collections;

namespace Click;

class CLI
{
	public String ProgramName;
	public Version? Version = null;
	public String About = null;

	public static CLI Context { get; private set; }
	public List<String> ErrorStack = new .() ~ DeleteContainerAndItems!(_);

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
		ErrorStack.Add(new .(error));
	}

	public void Report(String format, params Object[] args)
	{
		ErrorStack.Add(new String()..AppendF(format, params args));
	}

	public void PrintErrorStackTrace()
	{
		if (ErrorStack.IsEmpty)
			return;

		Console.WriteLine(ErrorStack[^1]);
		for (let error in ErrorStack[..<^1].Reversed)
			Console.WriteLine($"\n    Caused by: {error}");
	}

	public void Help()
	{
		scope HelpCommand().Run();
	}
}