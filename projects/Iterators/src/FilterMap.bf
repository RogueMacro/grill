using System;
using System.Collections;

namespace Iterators
{
	public struct FilterMapIterator<T, U> : IEnumerator<U>
	{
		private IEnumerator<T> Inner;
		private delegate Result<U>(T) Func;

		public this(IEnumerator<T> inner, delegate Result<U>(T) func)
		{
			Inner = inner;
			Func = func;
		}

		public Result<U> GetNext()
		{
			while ((Inner.GetNext() case .Ok(let val)))
				if (Func(val) case .Ok(let mapped))
					return .Ok(mapped);

			return .Err;
		}
	}
}