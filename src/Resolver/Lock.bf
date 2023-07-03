using System;
using System.Collections;

namespace Grill.Resolver;

typealias Lock = Dictionary<String, HashSet<Version>>;

//class Lock : Dictionary<String, HashSet<Version>>
//{
//	public ~this()
//	{
//		for (var (key, value) in this)
//		{
//			delete key;
//			delete value;
//		}
//	}
//}