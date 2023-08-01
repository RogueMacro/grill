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

	[SerializeField(Optional=true)]
	public Dictionary<String, Dependency> Dependencies ~ DeleteDictionaryAndItems!(_);

	[SerializeField(Optional=true)]
	public Dictionary<String, Feature> Features ~ DeleteDictionaryAndItems!(_);

	[Serializable]
	public class Package
	{
		public String Name ~ delete _;
		public Version Version;
		public String Description ~ delete _;

		[SerializeField(DefaultValue="true")]
		public bool Corlib = true;
	}

	//[Serializable]
	//public class Features
	//{
	//	public 
	//}

	[Serializable]
	public enum Feature : IHashable, IDisposable
	{
		case Project(String path);
		case List(List<String> features);

		public int GetHashCode()
		{
			switch (this)
			{
			case .Project(let path):
				return path.GetHashCode();
			case .List(let features):
				int hash = 0;
				for (let feature in features)
					hash *= feature.GetHashCode();
				return hash;
			}
		}

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

	public static Result<Self> FromPackage(String path)
	{
		let filePath = Path.InternalCombine(.. scope .(), path, Paths.MANIFEST_FILENAME);
		if (!File.Exists(filePath))
		{
			Errors.Report("Manifest not found");
			return .Err;
		}

		String file = scope .();
		Try!(File.ReadAllText(filePath, file)..Context("Failed to read manifest"));

		Serialize<Toml> serializer = scope .();
		return Try!(serializer.Deserialize<Manifest>(file)
			..Context(scope (str) => serializer.Error.ToString(str))
			..Context("Failed to deserialize manifest"));
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
		[SerializeField(Rename="Version")]
		public VersionReq Req;

		[SerializeField(Optional=true)]
		public HashSet<String> Features ~ DeleteContainerAndItems!(_);

		[SerializeField(DefaultValue="true")]
		public bool DefaultFeatures;
	}

	[Serializable]
	public class Local
	{
		public String Path ~ delete _;

		[SerializeField(Optional=true)]
		public HashSet<String> Features ~ DeleteContainerAndItems!(_);

		[SerializeField(DefaultValue="true")]
		public bool DefaultFeatures;
	}

	[Serializable]
	public class Git
	{
		[SerializeField(Rename="Git")]
		public String Url ~ delete _;

		public String Rev ~ delete _;

		[SerializeField(Optional=true)]
		public HashSet<String> Features ~ DeleteContainerAndItems!(_);

		[SerializeField(DefaultValue="true")]
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
