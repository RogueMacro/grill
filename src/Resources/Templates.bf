using System;

namespace Grill.Resources;

static class Templates
{
	public static IResource Manifest ~ delete _;
	public static IResource BeefProj ~ delete _;
	public static IResource BeefProjBinary ~ delete _;
	public static IResource Program ~ delete _;

	public static void Init()
	{
		Manifest = ResourceManager.Get("templates/Package.toml", => ManifestDefault);
		BeefProj = ResourceManager.Get("templates/BeefProj.toml", => BeefProjDefault);
		BeefProjBinary = ResourceManager.Get("templates/BeefProjBinary.toml", => BeefProjBinaryDefault);
		Program = ResourceManager.Get("templates/Program.bf", => ProgramDefault);
	}

	static IResource ManifestDefault() =>
		new InMemoryResource("Package.toml",
		"""
		[Package]
		Name = "$(Name)"
		Version = "0.1.0"
		Description = ""

		[Dependencies]
		""");

	static IResource BeefProjDefault() =>
		new InMemoryResource("BeefProj.toml",
		"""
		FileVersion = 1
		Dependencies = {}

		[Project]
		Name = "$(Name)"
		TargetType = "$(TargetType)"
		""");

	static IResource BeefProjBinaryDefault() =>
		new InMemoryResource("BeefProj.toml",
		"""
		FileVersion = 1
		Dependencies = {}

		[Project]
		Name = "$(Name)"
		StartupObject = "$(Namespace).Program"
		""");

	static IResource ProgramDefault() =>
		new InMemoryResource("Program.bf",
		"""
		using System;

		namespace $(Namespace)
		{
			class Program
			{
				public static int Main()
				{
					Console.WriteLine("Hello World!");
					return 0;
				}
			}
		}
		""");
}