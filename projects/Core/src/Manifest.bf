using System;
using System.Collections;
using System.IO;
using Serialize;
using Toml;
using SyncErr;
using Grill.Util;

namespace Grill;

[Serializable]
class Manifest
{
	public Package Package ~ delete _;

	[Serialize(.Optional)]
	public Workspace Workspace ~ delete _;

	[Serialize(.Optional)]
	public Dictionary<String, Dependency> Dependencies ~ DeleteDictionaryAndItems!(_);

	[Serialize(Default="new .")]
	public Features Features ~ delete _;

	[Serializable]
	public class Package
	{
		public String Name ~ delete _;
		public Version Version;
		public String Description ~ delete _;

		[Serialize(DefaultValue="true")]
		public bool Corlib = true;
	}

	[Serializable]
	public class Workspace
	{
		[Serialize(.Optional)]
		public String StartupProject ~ delete _;

		[Serialize(.Optional)]
		public List<String> Members ~ DeleteContainerAndItems!(_);
	}

	[Serializable]
	public class Features
	{
		[Serialize(.Optional)]
		public List<String> Default ~ DeleteContainerAndItems!(_);

		[Serialize(.Flatten, Default="new .")]
		public Dictionary<String, Feature> Optional ~ DeleteDictionaryAndItems!(_);
	}

	[Serializable]
	public enum Feature : IDisposable
	{
		case Project(String path);
		case List(List<String> features);

		public void Dispose()
		{
			switch (this)
			{
			case .Project(let path):
				delete path;
			case .List(let features):
				 DeleteContainerAndItems!(features);
			}
		}
	}

	public static Result<Self> FromPackage(StringView path)
	{
		if (!Directory.Exists(path))
			Bail!(scope $"No package found at {path}");

		let filePath = Path.InternalCombine(.. scope .(), path, Paths.MANIFEST_FILENAME);
		if (!File.Exists(filePath))
			Bail!(scope $"Manifest not found in {path}");

		String file = scope .();
		Try!(File.ReadAllText(filePath, file)
			..Context(scope (str) => str.AppendF($"Failed to read {filePath}")));

		Serializer<Toml> serializer = scope .();
		return Try!(serializer.Deserialize<Manifest>(file)
			..Context(scope (str) => serializer.Error.ToString(str))
			..Context(scope (str) => str.AppendF($"Failed to deserialize {filePath}")));
	}
}

[Serializable]
enum Dependency : IDisposable
{
	case Simple(VersionReq);
	case Advanced(Advanced);
	case Local(Local);
	case Git(Git);

	[Serializable]
	public class Advanced
	{
		[Serialize(Rename="Version")]
		public VersionReq Req;

		[Serialize(.Optional)]
		public HashSet<String> Features ~ DeleteContainerAndItems!(_);

		[Serialize(DefaultValue="true")]
		public bool DefaultFeatures;
	}

	[Serializable]
	public class Local
	{
		public String Path ~ delete _;

		[Serialize(Default="new .")]
		public HashSet<String> Features ~ DeleteContainerAndItems!(_);

		[Serialize(DefaultValue="true")]
		public bool DefaultFeatures;
	}

	[Serializable]
	public class Git
	{
		[Serialize(Rename="Git")]
		public String Url ~ delete _;

		public String Rev ~ delete _;

		[Serialize(.Optional)]
		public HashSet<String> Features ~ DeleteContainerAndItems!(_);

		[Serialize(DefaultValue="true")]
		public bool DefaultFeatures;
	}

	public void Dispose()
	{
		switch (this)
		{
		case .Simple: break;
		case .Advanced(let val): delete val;
		case .Local(let val): delete val;
		case .Git(let val): delete val;
		}
	}
}
