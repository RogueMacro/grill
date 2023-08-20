using System;
using System.IO;

namespace Grill.Resources;

class FileResource : IResource
{
	public String SourcePath ~ delete _;

	String fileName ~ delete _;

	public this(StringView path)
	{
		SourcePath = new .(path);
		fileName = Path.GetFileName(path, .. new .());
	}

	public Result<void> Place(StringView path, params (StringView, StringView)[] replace)
	{
		var path;
		if (!Path.HasExtension(path))
			path = Path.InternalCombine(.. scope:: .(), path, fileName);

		if (replace.IsEmpty)
			return File.Copy(SourcePath, path).Convert();

		String text = scope .();
		Try!(File.ReadAllText(SourcePath, text));
		for (let (key, value) in replace)
			text.Replace(key, value);

		return File.WriteAllText(path, text);
	}
}