using System;
using System.Collections;

namespace Iterators
{
	public struct MapIterator<T, U> : IEnumerator<U>
	{
		private IEnumerator<T> Inner;
		private delegate U(T) Func;

		public this(IEnumerator<T> inner, delegate U(T) func)
		{
			Inner = inner;
			Func = func;
		}

		public Result<U> GetNext()
		{
			if (Inner.GetNext() case .Ok(let val))
				return .Ok(Func(val));
			return .Err;
		}
	}
}