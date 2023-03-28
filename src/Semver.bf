using System;
using Serialize;
using Serialize.Implementation;

namespace Grill;

struct VersionReq : ISerializable
{
	public int Major;
	public int Minor;
	public int Patch;

	public void Serialize<S>(S serializer) where S : ISerializer
	{
		if (Major == -1)
			serializer.SerializeString("*");
		else
			serializer.SerializeString(scope $"^{Major}.{Minor}.{Patch}");
	}

	public static Result<Self> Deserialize<D>(D deserializer) where D : IDeserializer
	{
		let str = Try!(deserializer.DeserializeString());
		defer delete str;

		if (str == "*")
			return VersionReq() { Major = -1, Minor = -1, Patch = -1 };
		else if (str.StartsWith("^"))
		{
			str.Remove(0, 1);
			var comps = str.Split('.');
			VersionReq req = .();
			req.Major = int.Parse(comps.GetNext().Get());
			req.Minor = int.Parse(comps.GetNext().Get());
			req.Patch = int.Parse(comps.GetNext().Get());
			return req;
		}

		return VersionReq();
	}
}