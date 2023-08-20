using System;
using System.IO;

namespace Grill.Resources;

interface IResource
{
	Result<void> Place(StringView path, params (StringView, StringView)[] replace);
}