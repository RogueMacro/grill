using System;
using System.IO;

namespace Grill.Resources;

class InMemoryResource : IResource
{
	String fileName ~ delete _;
	String data ~ delete _;

	public this(StringView name, StringView text)
	{
		fileName = new .(name);
		data = new .(text);
	}

	public Result<void> Place(StringView path, params (StringView, StringView)[] replace)
	{
		String text = data;

		if (!replace.IsEmpty)
		{
			text = scope:: .(data);
			for (let (key, value) in replace)
				text.Replace(key, value);
		}	

		var path;
		if (!Path.HasExtension(path))
			path = Path.InternalCombine(.. scope:: .(), path, fileName);
		return File.WriteAllText(path, text);
	}
}