using System;
using System.Collections;
using System.IO;
using Serialize;
using Toml;

namespace Grill;

[Serializable]
class BeefSpace
{
	public int FileVersion = 1;

	[SerializeField(Optional=true)]
	public HashSet<String> Locked ~ DeleteContainerAndItems!(_);

	[SerializeField(Optional=true)]
	public Dictionary<String, ProjectEntry> Projects ~ DeleteDictionaryAndKeysAndValues!(_);

	[SerializeField(Optional=true)]
	public Dictionary<String, HashSet<String>> WorkspaceFolders ~
		{
			for (let set in _.Values)
				DeleteContainerAndItems!(set);
			DeleteDictionaryAndKeys!(_);
		};

	[SerializeField(Optional=true)]
	public WorkspaceSettings Workspace ~ delete _;

	[SerializeField(false)]
	String path ~ delete _;

	public void SetPath(StringView path)
	{
		delete this.path;
		this.path = new .(path);
	}

	public static Self CreateDefault(StringView? path = null)
	{
		//let proj = new Self() {
		//	Locked = new .(),
		//	Projects = new .(),
		//	WorkspaceFolders = new .(),
		//	Workspace = new .() {
		//		StartupProject = new .()
		//	}
		//};

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
		let ws = Try!(Toml.Deserialize<Self>(file));
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
		[SerializeField(Optional=true)]
		public String StartupProject ~ delete _;
	}
}

[Serializable]
class BeefProj
{
	public int FileVersion = 1;

	[SerializeField(Optional=true)]
	public Dictionary<String, String> Dependencies ~ DeleteDictionaryAndKeysAndValues!(_);

	[SerializeField(Optional=true)]
	public ProjectSettings Project ~ delete _;

	[SerializeField(false)]
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
		let proj = Try!(Toml.Deserialize<Self>(file));
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

		[SerializeField(DefaultValue=".BeefConsoleApplication")]
		public TargetType TargetType;

		[SerializeField(Optional=true)]
		public String StartupObject ~ delete _;

		[SerializeField(Optional=true)]
		public HashSet<String> ProcessorMacros ~ DeleteContainerAndItems!(_);

		[Serializable]
		public enum TargetType
		{
			BeefConsoleApplication,
			BeefLib,
			BeefGUIApplication
		}
	}
}