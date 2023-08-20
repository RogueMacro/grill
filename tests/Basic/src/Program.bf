using System;
using Grill;

namespace Basic
{
	class Program
	{
		public static int Main()
		{
			if (Test() case .Err)
				return -1;
			return 0;
		}

		static Result<void> Test()
		{
			Package package = scope .();
			Try!(package.Open("bare"));
			Try!(package.Make());

			return .Ok;
		}
	}
}