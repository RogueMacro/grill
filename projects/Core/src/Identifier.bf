using System;

namespace Grill;

class Identifier : RefCounted
{
	public String Name ~ delete _;
	public VersionIdent Version;

	public readonly String Str = new .() ~ delete _;

	public enum VersionIdent
	{
		case SemVer(Version version);
		case Git(String rev);
	}

	public this(StringView name, Version version)
	{
		Name = new .(name);
		Version = .SemVer(version);

		ToString(Str);
	}

	public override void ToString(String strBuffer)
	{
		strBuffer.Append(Name);
		strBuffer.Append('-');
		switch (Version)
		{
		case .SemVer(let v):
			strBuffer.Append(v);
		case .Git(let rev):
			strBuffer.Append(rev);
		}
	}	
}