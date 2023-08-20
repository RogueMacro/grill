using System;
using System.Collections;
using System.IO;
using Serialize;
using Toml;
using Grill.Util;

namespace Grill.Beef;

[Serializable]
class BeefSpace
{
	public int FileVersion = 1;

	[Serialize(.Optional)]
	public HashSet<String> Locked ~ DeleteContainerAndItems!(_);

	[Serialize(.Optional)]
	public Dictionary<String, ProjectEntry> Projects ~ DeleteDictionaryAndKeysAndValues!(_);

	[Serialize(.Optional)]
	public Dictionary<String, HashSet<String>> WorkspaceFolders ~
		{
			if (_ != null)
			{
				for (let set in _.Values)
					DeleteContainerAndItems!(set);
				DeleteDictionaryAndKeys!(_);
			}
		};

	[Serialize(.Optional)]
	public WorkspaceSettings Workspace ~ delete _;

	[Serialize(.Optional)]
	public Configs Configs ~ delete _;

	[Serialize(.Flatten)]
	Dictionary<String, Variant> _ ~ DeleteDictionaryAndItems!(_);

	[Serialize(.Skip)]
	String path ~ delete _;

	public void SetPath(StringView path)
	{
		delete this.path;
		this.path = new .(path);
	}

	public static Self CreateDefault(StringView? path = null)
	{
		let proj = new Self();

		if (path != null)
			proj.SetPath(path.Value);

		return proj;
	}

	public static Result<Self> FromPackage(StringView path)
	{
		let filePath = Path.InternalCombine(.. scope .(), path, "BeefSpace.toml");
		return Read(filePath);
	}

	public static Result<Self> Read(StringView path)
	{
		String file = scope .();
		Try!(File.ReadAllText(path, file));

		Serializer<Toml> serializer = scope .();
		let ws = Try!(serializer.Deserialize<Self>(file)
					..Context(scope (str) => serializer.Error.ToString(str))
					..Context(scope (str) => str.AppendF($"Failed to deserialize file: {path}")));

		ws.SetPath(path);
		return ws;
	}

	public Result<void> Save()
	{
		if (path == null)
			return (.Err)..Context("No path specified for BeefSpace");

		String buffer = scope .();
		Toml.Serialize(this, buffer);
		return File.WriteAllText(path, buffer);
	}

	[Serializable]
	public class ProjectEntry
	{
		public String Path ~ delete _;
	}

	[Serializable]
	public class WorkspaceSettings
	{
		[Serialize(.Optional)]
		public String StartupProject ~ delete _;
	}

	[Serializable]
	public class Configs
	{
		[Serialize(.Optional)] public Config Debug ~ delete _;
		[Serialize(.Optional)] public Config Release ~ delete _;
		[Serialize(.Optional)] public Config Test ~ delete _;

		[Serialize(.Flatten)]
		public Dictionary<String, Config> Other ~ DeleteDictionaryAndKeysAndValues!(_);

		[Serializable]
		public class Config
		{
			[Serialize(.Optional)] public PlatformConfig Win32 ~ delete _;
			[Serialize(.Optional)] public PlatformConfig Win64 ~ delete _;

			[Serialize(.Flatten)]
			public Dictionary<String, PlatformConfig> OtherPlatforms ~ DeleteDictionaryAndKeysAndValues!(_);

			[Serializable]
			public class PlatformConfig
			{
				[Serialize(.Optional, DefaultValue="true")] public bool Enabled;
				[Serialize(.Optional, DefaultValue="new .(\"Test\")")] public String Config ~ delete _;
				[Serialize(.Optional)] public String Platform ~ delete _;
			}
		}
	}
}