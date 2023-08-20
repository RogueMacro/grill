using System;
using System.Collections;
using Grill.Resolution;

namespace Grill.Resolution;

/// A cache of previously loaded packages.
class RegistryCache : RefCounted
{
	Dictionary<String, RefCounted<PackageMetadata>> cache = new .() ~ {
		for (let value in _.Values)
			value.Release();
		DeleteDictionaryAndKeys!(_);
	};

	RefCounted<IRegistry> registry ~ _.Release();

	public this(RefCounted<IRegistry> registry)
	{
		this.registry = registry..AddRef();
	}

	/// Get a package from the cache or load it from the registry.
	/// Package metadata will not outlive the package cache.
	public Result<PackageMetadata> GetPackage(String name)
	{
		return Try!(GetPackageRef(name))..Release().Value;
	}

	/// Get a reference-counted PackageMetadata.
	public Result<RefCounted<PackageMetadata>> GetPackageRef(String name)
	{
		RefCounted<PackageMetadata> packageRef;
		if (cache.GetValue(name) case .Ok(let val))
		{
			packageRef = val..AddRef();
		}
		else
		{
			let package = Try!(registry->GetPackage(name));
			packageRef = RefCounted<PackageMetadata>.Attach(package);
			cache[new .(name)] = packageRef..AddRef();
		}
		
		return packageRef;
	}

	/// Returns true if the package exists in the registry.
	/// Will load the package if not already cached.
	public bool ContainsPackage(String str)
	{
		if (cache.ContainsKey(str))
			return true;

		return GetPackage(str) case .Ok;
	}
}