using System;
using System.IO;
using Grill.Resources;

namespace Grill.CLI;

class ResourceProvider : IResourceProvider
{
	public static readonly String AbsolutePath ~ delete _;

	public static this()
	{
		let exe = Environment.GetExecutableFilePath(.. scope .());
		AbsolutePath = Path.GetDirectoryPath(exe, .. new .());
		Path.InternalCombine(AbsolutePath, "resources");
	}

	public void GetFullPath(StringView path, String target)
	{
		Path.InternalCombine(target, AbsolutePath, path);
	}

	public Result<IResource> Get(StringView path)
	{
		let fullPath = GetFullPath(path, .. scope .());
		if (!File.Exists(fullPath))
			return .Err;
		return new FileResource(fullPath);
	}
}