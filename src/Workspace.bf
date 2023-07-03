using System;
using System.Collections;
using System.IO;
using System.Threading;
using System.Threading.Tasks;
using Grill.Console;
using Grill.Resolver;
using Click;
using Toml;
using Serialize;
using System.Diagnostics;

namespace Grill;

class Workspace
{
	public Manifest Manifest ~ delete _;
	public Lock Lock ~ DeleteDictionaryAndKeysAndValues!(_);

	public Packages Packages ~ delete _;

	String path ~ delete _;

	RefCounted<RegistryCache> cache ~ if (_ != null) _.Release();

	public this(StringView path)
	{
		this.path = new .(path);
	}

	public Result<void> Open()
	{
		let manifestPath = Path.InternalCombine(.. scope .(), path, Paths.MANIFEST_FILENAME);
		if (!File.Exists(manifestPath))
		{
			CLI.Context.Report("Manifest not found");
			return .Err;
		}

		String file = scope .();
		Try!(File.ReadAllText(manifestPath, file)..Context("Failed to read manifest"));

		Serialize<Toml> serializer = scope .();
		Manifest = Try!(serializer.Deserialize<Manifest>(file)
			..Context(scope (str) => serializer.Error.ToString(str))
			..Context($"Failed to deserialize manifest"));

		let lockPath = Path.InternalCombine(.. scope .(), path, Paths.LOCK_FILENAME);
		if (File.Exists(lockPath))
		{
			file.Clear();
			Try!(File.ReadAllText(lockPath, file)..Context("Failed to read lock file"));

			Lock = Try!(serializer.Deserialize<Lock>(file)
				..Context(scope (str) => serializer.Error.ToString(str))
				..Context($"Failed to deserialize lock file"));
		}

		Packages = new .(Path.InternalCombine(.. scope .(), path, Paths.PACKAGE_DIRECTORY));

		return .Ok;
	}

	public Result<void> Make(bool quiet = false)
	{
		GConsole.Quiet = quiet;

		//if (quiet)
		//{
		//	Try!(UpdateStep());
		//	// Resolve
		//	Try!(FetchStep(null));
		//	// Build workspace
		//}

		{
			MultiProgress multi = scope .();
			defer multi.Finish();
			Log.SetProgress(multi);

			GConsole.WriteLine($"        {Styled("Make")..Bright()..Cyan()} {Manifest.Package.Name} v{Manifest.Package.Version}");
			Log.SetPosHere();
			GConsole.WriteLine();

			multi.SetBaselineHere();

			Try!(Step(multi,
				 scope $"       {Styled("[1/4]")..Bold()..Dim()} 🧭 Updating ",
				 scope $"       {Styled("[1/4]")..Bold()..Dim()} 🧭 Up to date ", 
				 scope => Update)
				 ..Context("Failed to update registry"));

			Try!(Step(multi,
				 scope $"       {Styled("[2/4]")..Bold()..Dim()} 🔍 Resolving ",
				 scope $"       {Styled("[2/4]")..Bold()..Dim()} 🔍 Resolution ready ",
				 scope => Resolve)
				 ..Context("Failed to resolve dependencies"));

			Try!(Step(multi,
				 scope $"       {Styled("[3/4]")..Bold()..Dim()} 🚚 Fetching ",
				 scope $"       {Styled("[3/4]")..Bold()..Dim()} 🚚 Packages on disk ", 
				 scope () => Fetch(multi))
				 ..Context("Failed to fetch packages"));

			Try!(Step(multi,
				 scope $"       {Styled("[4/4]")..Bold()..Dim()} 📦 Building ",
				 scope $"       {Styled("[4/4]")..Bold()..Dim()} 📦 Workspace done ",
				 scope => Build)
				 ..Context("Failed to build workspace"));
		}

		GConsole.WriteLine("             🍝 Enjoy your spaghetti!");

		return .Ok;
	}

	Result<void> Update()
	{
		PathRegistry registry = new .("https://github.com/roguemacro/grill-index");
		RefCounted<IRegistry> registryRef = .Attach(registry);
		cache = .Attach(new .(registryRef));
		registry.Fetch();
		registryRef.Release();

		return .Ok;
	}

	Result<void> Resolve()
	{
		let previousLock = Lock;
		Resolver resolver = scope .(cache);
		Lock = Try!(resolver.Resolve(Manifest));
		DeleteDictionaryAndKeysAndValues!(previousLock);
		
		String s = new .();
		defer delete s;
		Toml.Serialize(Lock, s);

		String filePath = Path.InternalCombine(.. scope .(), path, Paths.LOCK_FILENAME);
		return File.WriteAllText(filePath, s)..Context("Failed to write lock file");
	}

	Result<void> Fetch(MultiProgress multi)
	{
		List<(String, Version)> pkgs = scope .();
		for (let (pkg, versions) in Lock)
			for (let version in versions)
				pkgs.Add((pkg, version));

		ProgressBar progress = scope .(pkgs.Count);
		multi.Add(progress);
		defer multi.Remove(progress);

		progress.Tick();

		for (let (pkg, version) in pkgs)
		{
			progress.UpdateText($"{pkg} 0%");

			let package = Try!(Packages.Install(pkg, version, cache, scope (stats) => {
				let percent = Math.Floor((float)stats.indexed_objects / stats.total_objects * 100);
				progress.UpdateText($"{pkg} {percent}%");
			})
				..Context($"Failed to install {pkg} v{version}"));

			progress.Text.Set("");
			progress.Tick();

			if (package.JustInstalled)
				Log.Print(Styled("Fetched")..Bright()..Green(), "{} v{}", pkg, version);
		}

		progress.Finish();
		return .Ok;
	}

	Result<void> Build()
	{
		//Thread.Sleep(1000);
		return .Ok;
	}

	Result<void> Step(MultiProgress multi, StringView message, StringView finish, delegate Result<void>() func)
	{
		ProgressSpinner progress = new .(message, finish);
		multi.Add(progress);
		progress.EnableSteadyTick(200);

		let result = func();

		progress.Finish();
		return result;
	}
}