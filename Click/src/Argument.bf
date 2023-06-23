using System;

namespace Click;

class Argument
{
	public String Name ~ delete _;
	public ArgumentType Type;

	public this(StringView name, ArgumentType type)
	{
		Name = new .(name);
		Type = type;
	}

	public ~this()
	{
		if (Type case .Value(let s))
			delete s;
	}
}

enum ArgumentType
{
	case Positional;
	case Flag;
	case Value(String);
}