using System;
using System.Collections;

namespace Grill.Util;

static
{
	public static mixin ClearAndDeletePairs(var container)
	{
		if (container != null)
		{
			for (let pair in container)
			{
				delete pair.key;
				delete pair.value;
			}

			container.Clear();
		}
	}

	public static mixin DeleteDictionaryAndItems<K, V>(Dictionary<K, V> container)
		where K : IHashable, delete
		where V : IDisposable
	{
		if (container != null)
		{
			for (var value in container.Values)
				value.Dispose();
			DeleteDictionaryAndKeys!(container);
		}
	}
}