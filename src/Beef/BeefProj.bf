using System;
using System.Collections;
using System.IO;
using Serialize;
using Toml;
using Grill.Util;

namespace Grill.Beef;

[Serializable]
class BeefProj
{
	public int FileVersion = 1;

	[Serialize(.Optional)]
	public Dictionary<String, String> Dependencies ~ DeleteDictionaryAndKeysAndValues!(_);

	[Serialize(.Optional)]
	public ProjectSettings Project ~ delete _;

	[Serialize(.Flatten)]
	Dictionary<String, Variant> _ ~ DeleteDictionaryAndItems!(_);

	[Serialize(.Skip)]
	String path ~ delete _;

	public void SetPath(StringView path)
	{
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
		let filePath = Path.InternalCombine(.. scope .(), path, "BeefProj.toml");
		return Read(filePath);
	}

	public static Result<Self> Read(StringView path)
	{
		String file = scope .();
		Try!(File.ReadAllText(path, file));
		
		Serializer<Toml> serializer = scope .();
		let proj = Try!(serializer.Deserialize<Self>(file)
					..Context(scope (str) => serializer.Error.ToString(str))
					..Context(scope (str) => str.AppendF($"Failed to deserialize file: {path}")));

		proj.SetPath(path);
		return proj;
	}

	public Result<void> Save()
	{
		if (path == null)
			return (.Err)..Context("No path specified for BeefProj");

		String buffer = scope .();
		Toml.Serialize(this, buffer);
		return File.WriteAllText(path, buffer);
	}

	[Serializable]
	public class ProjectSettings
	{
		public String Name ~ delete _;

		[Serialize(DefaultValue=".Library")]
		public TargetType TargetType;

		[Serialize(.Optional)]
		public String StartupObject ~ delete _;

		[Serialize(.Optional)]
		public HashSet<String> ProcessorMacros ~ DeleteContainerAndItems!(_);
	}
}