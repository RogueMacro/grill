using System;
using System.Reflection;

namespace Click;

[AttributeUsage(.Class, .ReflectAttribute, AlwaysIncludeUser = .All, ReflectUser = .All)]
struct CommandAttribute : Attribute, IOnTypeInit
{
	public String Name;
	public String About;

	public this(String name, String about = null)
	{
		Name = name;
		About = about;
	}

	[Comptime]
	public void OnTypeInit(Type type, Self* prev)
	{
		Compiler.EmitAddInterface(type, typeof(ICommand));
	}
}

typealias ArgAttribute = ArgumentAttribute;

[AttributeUsage(.Field, .ReflectAttribute)]
struct ArgumentAttribute : Attribute
{
	public String Name;
	public String About;
	public char8? Short;
	public String Default;
	public bool Required;

	public this(String name = null, String about = null, String short = null, String _default = null, bool required = false)
	{
		if (short?.Length > 1)
			Runtime.FatalError("Argument short name can only be one character long");

		Name = name;
		About = about;
		Short = short?[0];
		Default = _default;
		Required = required;
	}
}