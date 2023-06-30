using System;
using System.Collections;
using Serialize;
using Toml;

namespace Grill;

[Serializable]
class Manifest
{
	public Package Package ~ delete _;
	public Dictionary<String, Dependency> Dependencies ~ DeleteDictionaryAndKeys!(_);

	[Serializable]
	public class Package
	{
		public String Name ~ delete _;
		public Version Version;
		public String Description ~ delete _;
	}
}

[Serializable]
enum Dependency
{
	case Simple(VersionReq);
}

[Serializable]
class Advanced
{
	public VersionReq Version;
}
