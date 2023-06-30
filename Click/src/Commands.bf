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
		{
			CLI.Context.Report($"Unknown subcommand {cmd}");
			return .Err;
		}

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
			let argName = attr.Name ?? field.Name;
			bool valueSet = false;

			if (field.FieldType == typeof(bool))
			{
				for (let arg in args)
				{
					let flagName = StringView(arg)..TrimStart('-');
					if (arg.StartsWith("--") && flagName == argName)
					{
						field.SetValue(command, true);
						args.Remove(arg);
						valueSet = true;
						break;
					}
					else if (attr.Short != null && arg.StartsWith("-") && flagName.Length == 1 && flagName[0] == attr.Short)
					{
						field.SetValue(command, true);
						args.Remove(arg);
						valueSet = true;
						break;
					}
				}
			}
			else if (field.FieldType == typeof(String))
			{
				var argIter = args.GetEnumerator();
				for (let arg in argIter)
				{
					let flagName = StringView(arg)..TrimStart('-');
					if (arg.StartsWith("--") && flagName == argName)
					{
						if (argIter.GetNext() case .Ok(let nextArg))
						{
							field.SetValue(command, new String(nextArg));
							args.Remove(arg);
							args.Remove(nextArg);
							valueSet = true;
							break;
						}
						else
						{
							CLI.Context.Report($"Missing value after {arg}");
							return .Err;
						}
					}
					else if (attr.Short != null && arg.StartsWith("-") && flagName.Length == 1 && flagName[0] == attr.Short)
					{
						if (argIter.GetNext() case .Ok(let nextArg))
						{
							field.SetValue(command, new String(nextArg));
							args.Remove(arg);
							args.Remove(nextArg);
							valueSet = true;
							break;
						}
						else
						{
							CLI.Context.Report($"Missing value after {arg}");
							return .Err;
						}
					}
				}

				if (!valueSet)
				{
					for (let arg in args)
					{
						if (!arg.StartsWith('-'))
						{
							args.Remove(arg);
							var arg;
							String arg = new .(arg);
							arg.Trim('"');
							field.SetValue(command, arg);
							valueSet = true;
							break;
						}
					}
				}

				if (!valueSet && attr.Default != null)
				{
					field.SetValue(command, new String(attr.Default));
					valueSet = true;
				}

				if (!valueSet && attr.Required)
				{
					CLI.Context.Report($"Argument '{argName}' is required");
					return .Err;
				}
			}
		}

		return command.Run();
	}
}