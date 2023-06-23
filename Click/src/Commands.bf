using System;
using System.Collections;

namespace Click;

class Commands : Dictionary<StringView, Type>
{
	public void RegisterAll()
	{
		for (let type in Type.Types)
		{
			if (type.GetCustomAttribute<CommandAttribute>() case .Ok(let attr))
				Add(attr.Name, type);
		}
	}

	public void Register<T>(StringView name) where T : ICommand
	{
		Add(name, typeof(T));
	}

	public Result<void> Dispatch(StringView cmd, Span<StringView> _args)
	{
		if (!ContainsKey(cmd))
			return .Err;

		List<StringView> args = scope .();
		for (let a in _args) args.Add(a);

		let type = this[cmd];
		ICommand command = (.)Try!(type.CreateObject());
		defer delete command;
		for (let field in type.GetFields())
		{
			if (field.GetCustomAttribute<ArgumentAttribute>() case .Err)
				continue;

			let attr = field.GetCustomAttribute<ArgumentAttribute>().Value;
			if (field.FieldType == typeof(bool))
			{
				for (let arg in args)
				{
					let flagName = StringView(arg)..TrimStart('-');
					if (arg.StartsWith("--") && flagName == attr.Name)
					{
						field.SetValue(command, true);
						args.Remove(arg);
					}
				}
			}
			else if (field.FieldType == typeof(String))
			{
				if (args.Count > 0 && !args[0].StartsWith('-'))
					field.SetValue(command, new String(args.PopBack()));
			}
		}

		return command.Run();
	}
}