using System;
using System.Collections;

namespace Grill.Resources;

static class ResourceManager
{
	static IResourceProvider provider ~ delete _;

	public static void Init<T>() where T : IResourceProvider, new, class
	{
		provider = new T();
		Init();
	}

	public static void Init()
	{
		Templates.Init();
	}

	public static IResource Get(StringView path, function IResource() defaultTo)
	{
		if (provider != null && provider.Get(path) case .Ok(let res))
			return res;
		return defaultTo();
	}
}