using System;
using Serialize;

namespace Grill.Beef;

[Serializable]
public enum TargetType
{
	[Serialize(Rename="BeefConsoleApplication")]
	Binary,
	[Serialize(Rename="BeefLib")]
	Library,
	[Serialize(Rename="BeefGUIApplication")]
	GUI,
	[Serialize(Rename="BeefTest")]
	Test
}

extension TargetType
{
	public override void ToString(String strBuffer)
	{
		switch (this)
		{
		case .Binary: strBuffer.Append("BeefConsoleApplication");
		case .Library: strBuffer.Append("BeefLib");
		case .GUI: strBuffer.Append("BeefGUIApplication");
		case .Test: strBuffer.Append("BeefTest");
		}
	}
}