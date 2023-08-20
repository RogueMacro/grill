using System;
using System.Collections;

namespace Iterators
{
	public struct ChainedIterator<T> : IEnumerator<T>
	{
		private IEnumerator<T> First;
		private IEnumerator<T> Second;

		public this(IEnumerator<T> first, IEnumerator<T> second)
		{
			First = first;
			Second = second;
		}

		public Result<T> GetNext()
		{
			if (First.GetNext() case .Ok(let val))
				return val;
			return Second.GetNext();
		}
	}
}