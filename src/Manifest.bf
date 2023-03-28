using System;
using System.Collections;
using Serialize;
using Toml;

namespace Grill;

[Serializable]
class Manifest
{
	public Package Package ~ delete _;
	public Dictionary<String, String> Dependencies ~ DeleteDictionaryAndKeysAndValues!(_);
}

[Serializable]
class Package
{
	public String Name ~ delete _;
	public String Version ~ delete _;
	public String Description ~ delete _;
}