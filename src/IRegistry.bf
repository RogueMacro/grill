using System;
using System.Collections;
using System.IO;
using Serialize;
using Toml;

using static BuildTools.Git.Git;

namespace Grill;

interface IRegistry
{
	/// Get package metadata from registry.
	/// Returns .Err if package doesn't exist.
	Result<PackageMetadata> GetPackage(StringView name);
}

[Serializable]
public class PackageMetadata
{
	public String url ~ delete _;
	public String description ~ delete _;
	public Dictionary<Version, VersionMetadata> versions ~ DeleteDictionaryAndValues!(_);
}

[Serializable]
public class VersionMetadata
{
	public String rev ~ delete _;
	public Dictionary<String, VersionReq> deps ~ DeleteDictionaryAndKeys!(_);
}