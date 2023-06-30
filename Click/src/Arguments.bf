using System;
using System.Collections;

namespace Click;

class Arguments : List<StringView>
{
	public this(StringView input)
	{
		Parse(input);
	}

	[AllowAppend]
	public this(Span<StringView> args) : base(args)
	{

	}

	public Result<(StringView cmd, Span<StringView> args)> Subcommand()
	{
		if (IsEmpty)
			return .Err;

		let firstArg = this[0];
		if (firstArg.IsEmpty || !firstArg[0].IsLetter)
			return .Err;
		
		return (firstArg, GetRange(1));
	}

	private Result<void> Parse(StringView input)
	{
		var input;
		input.Trim();

		int mark = 0;
		bool isString = false;
		for (int i = 0; i < input.Length; i++)
		{
			let c = input[i];
			if (c == '\\')
			{
				i++;
			}
			else if (c == '"')
			{
				if (isString)
					EndArg!();
				isString = !isString;
				mark = i + 1;
			}
			else if (c == ' ')
			{
				EndArg!();
				mark = i + 1;
			}
		}

		Add(input.Substring(mark));

		mixin EndArg()
		{
			Add(input.Substring(mark, i - mark));
			while (i < input.Length - 1 && input[i + 1] == ' ')
				i++;
		}

		return .Ok;
	}
}