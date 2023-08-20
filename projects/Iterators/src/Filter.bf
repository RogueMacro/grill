using System;
using System.Collections;

namespace Iterators
{
	public struct FilterIterator<T> : IEnumerator<T>
	{
		private IEnumerator<T> Inner;
		private delegate bool(T) Predicate;

		public this(IEnumerator<T> inner, delegate bool(T) predicate)
		{
			Inner = inner;
			Predicate = predicate;
		}

		public Result<T> GetNext()
		{
			while ((Inner.GetNext() case .Ok(let val)))
				if (Predicate(val))
					return .Ok(val);

			return .Err;
		}
	}
}