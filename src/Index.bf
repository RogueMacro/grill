using System;
using System.Collections;
using System.IO;
using Serialize;
using Toml;

using static BuildTools.Git.Git;

namespace Grill;

[Serializable]
class IndexManager
{
	public static Result<Index> Index => _index ?? Load();
	private static Index _index ~ DeleteDictionaryAndKeysAndValues!(_);

	public static Result<Index> Load()
	{
		if (!File.Exists("tmp/index.toml"))
			Update();

		String file = scope .();
		Try!(File.ReadAllText("tmp/index.toml", file));

		let serializer = scope Serialize<Toml>();
		let result = serializer.Deserialize<Index>(file);
		if (result case .Err)
		{
			Console.WriteLine("Error: {}", serializer.Error);
			return .Err;
		}

		_index = result.Value;
		return result;
	}

	public static Result<void> Update()
	{
		Try!(Directory.DelTree("tmp/"));

		git_repository* repo = null;
		char8* url = "https://github.com/RogueMacro/grill-index";
		char8* path = "tmp/";
		if (git_clone(out repo, url, path, null) != .GIT_OK)
			return .Err;

		return .Ok;
	}

	typealias Index = Dictionary<String, IndexEntry>;

	[Serializable]
	public class IndexEntry
	{
		public String url ~ delete _;
		public String description ~ delete _;
		public Dictionary<String, VersionMetadata> versions ~ DeleteDictionaryAndKeysAndValues!(_);
	}

	[Serializable]
	public class VersionMetadata
	{
		public String rev ~ delete _;
		public Dictionary<String, VersionReq> deps ~ DeleteDictionaryAndKeys!(_);
	}
}