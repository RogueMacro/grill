using System;

namespace Click;

[AttributeUsage(.Class, .ReflectAttribute, AlwaysIncludeUser=.All, ReflectUser=.All)]
struct CommandAttribute : Attribute, IOnTypeInit
{
	public String Name;
	public String _about;

	public String About { get => _about; set mut { _about = value; } }

	public this(String name, String about = null)
	{
		Name = name;
		_about = about;
	}

	[Comptime]
	public void OnTypeInit(Type type, Self* prev)
	{
		Compiler.EmitAddInterface(type, typeof(ICommand));
	}
}

[AttributeUsage(.Class, .ReflectAttribute, AlwaysIncludeUser=.All, ReflectUser=.All)]
struct AttrAttribute : Attribute
{
	String _about = null;
	
	public String About { get => _about; set mut { _about = value; } }

	public this()
	{
		
	}
}

typealias ArgAttribute = ArgumentAttribute;

[AttributeUsage(.Field, .ReflectAttribute)]
struct ArgumentAttribute : Attribute
{
	public String Name = null;

	public bool Required = false;

	public this()
	{

	}

	public this(String name)
	{
		Name = name;
	}
}