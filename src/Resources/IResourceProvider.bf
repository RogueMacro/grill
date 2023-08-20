using System;
using System.IO;

namespace Grill.Resources;

interface IResourceProvider
{
	Result<IResource> Get(StringView path);
}