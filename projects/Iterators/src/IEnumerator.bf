using System;
using System.Collections;

namespace Iterators
{
    public static
	{
		public static FilterIterator<T> Filter<T>(this IEnumerator<T> enumerator, delegate bool(T) func) => FilterIterator<T>(enumerator, func);
		public static MapIterator<T, U> Map<T, U>(this IEnumerator<T> enumerator, delegate U(T) func) => MapIterator<T, U>(enumerator, func);
		public static FilterMapIterator<T, U> FilterMap<T, U>(this IEnumerator<T> enumerator, delegate Result<U>(T) func) => FilterMapIterator<T, U>(enumerator, func);
		public static ChainedIterator<T> Chain<T, E>(this IEnumerator<T> enumerator, E other) where E : IEnumerator<T> => ChainedIterator<T>(enumerator, other);
		public static ChainedIterator<T> Chain<T, E>(this IEnumerator<T> enumerator, E other) where E : IEnumerable<T> => ChainedIterator<T>(enumerator, other.GetEnumerator());

		public static bool Any<T>(this IEnumerator<T> enumerator, delegate bool(T) predicate)
		{
			while (enumerator.GetNext() case .Ok(let val))
				if (predicate(val))
					return true;

			return false;
		}

		public static void Collect<T, C>(this IEnumerator<T> enumerator, C collection)
			where C : concrete, ICollection<T>
		{
			while (enumerator.GetNext() case .Ok(let val))
				collection.Add(val);
		}
	}
}
    