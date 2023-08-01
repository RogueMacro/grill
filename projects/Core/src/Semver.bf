using System;
using Serialize;
using Serialize.Implementation;

namespace System
{
	extension Version : ISerializableKey, ISerializeAsPrimitive
	{
		public void Serialize<S>(S serializer) where S : ISerializer
		{
			let str = ToString(..scope .());
			serializer.SerializeString(str);
		}

		public static Result<Self> Deserialize<D>(D deserializer) where D : IDeserializer
		{
			let str = Try!(deserializer.DeserializeString());
			defer delete str;
			return Version.Parse(str);
		}

		public void ToKey(String buffer)
		{
			ToString(buffer);
		}

		public override void ToString(String strBuffer)
		{
			strBuffer.AppendF($"{Major}.{Minor}.{Build}");
		}

		public int GetHashCode()
		{
			int hash = 17;
			hash = hash * 23 + Major.GetHashCode();
			hash = hash * 23 + Minor.GetHashCode();
			hash = hash * 23 + Build.GetHashCode();
			return hash;
		}
	}
}

namespace Grill;

struct VersionReq : ISerializable, ISerializeAsPrimitive
{
	public int Major;
	public int Minor;
	public int Patch;

	public bool Matches(Version version)
	{
		if (Major == -1)
			return true;

		return
			version.Major == Major &&
			(version.Minor > Minor ||
			(version.Minor == Minor &&
			version.Build >= Patch));
	}

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
		return Parse(str);
	}

	public static Result<Self> Parse(StringView str)
	{
		var str;
		str = scope String(str);
		if (str == "*")
			return VersionReq() { Major = -1, Minor = -1, Patch = -1 };
		else if (str.StartsWith("^") || str[0].IsDigit)
		{
			str.TrimStart('^');
			var comps = str.Split('.');
			VersionReq req = .();
			req.Major = Try!(int.Parse(Try!(comps.GetNext())));
			req.Minor = Try!(int.Parse(Try!(comps.GetNext())));
			if (comps.GetNext() case .Ok(let comp))
				req.Patch = Try!(int.Parse(comp));
			else
				req.Patch = 0;
			return req;
		}

		return .Err;
	}
}