using System;
using System.Collections;
using System.IO;
using System.Threading;
using System.Threading.Tasks;

using Grill.Beef;
using Grill.Console;
using Grill.Resolution;
using Grill.Resources;

using Serialize;
using Toml;
using SyncErr;

namespace Grill;

class Package
{
	public Manifest Manifest ~ delete _;
	public Lock Lock ~ DeleteDictionaryAndKeysAndValues!(_);
	public Packages Packages ~ delete _;

	String path = new .() ~ delete _;
	RegistryCache cache ~ if (_ != null) _.ReleaseLastRef();
	RefCounted<IRegistry> registry ~ if (_ != null) _.Release();

	bool isOpen = false;

	const StringView[?] RESERVED_DIRS = .("src", "pkg", "build", "recovery", "tests", "common");

	/// Open a workspace.
	public Result<void> Open(StringView path, StringView? pkgs = null)
	{
		if (isOpen)
			Bail!("Workspace is already open");

		this.path.Set(path);

		Manifest = Try!(decltype(Manifest).FromPackage(path));

		let lockPath = Path.InternalCombine(.. scope .(), path, Paths.LOCK_FILENAME);
		if (File.Exists(lockPath))
		{
			String file = scope .();
			Try!(File.ReadAllText(lockPath, file)..Context("Failed to read lock file"));

			Serializer<Toml> serializer = scope .();
			Lock = Try!(serializer.Deserialize<Lock>(file)
				..Context(scope (str) => serializer.Error.ToString(str))
				..Context($"Failed to deserialize lock file"));
		}

		Packages = new .(Path.InternalCombine(.. scope .(), pkgs ?? path, Paths.PACKAGE_DIRECTORY));

		isOpen = true;
		return .Ok;
	}

	/// Create a workspace.
	public Result<void> Create(StringView path, StringView name = "", TargetType targetType = .Binary)
	{
		if (!Directory.IsEmpty(path))
			Bail!("Directory is not empty");

		if (!Directory.Exists(path))
			Directory.CreateDirectory(path);

		var name;
		if (name.IsEmpty)
		{
			let dir = Path.GetFullPath(scope .(path), .. scope .());
			name = Path.GetFileName(dir, .. scope:: .());
		}

		Try!(Templates.Manifest.Place(path, ("$(Name)", name)));

		let ns = scope String(name)..Replace('-', '_');

		if (targetType case .Binary)
		{
			Try!(Templates.BeefProjBinary.Place(
				path,
				("$(Name)", name),
				("$(Namespace)", ns)
			));
		}
		else
		{
			Try!(Templates.BeefProj.Place(
				path,
				("$(Name)", name),
				("$(TargetType)", targetType.ToString(.. scope .()))
			));
		}

		let src = Path.InternalCombine(.. scope .(), path, "src");
		Directory.CreateDirectory(src);
		Try!(Templates.Program.Place(src, ("$(Namespace)", ns)));

		return Open(path);
	}

	/// Create an integration test.
	public Result<void> CreateIntegrationTest(StringView name)
	{
		AssertOpen!();

		if (RESERVED_DIRS.Contains(name))
			Bail!(scope $"{name} is reserved. Choose something else.");

		let tests = Path.InternalCombine(.. scope .(), path, Paths.TEST_DIRECTORY);
		if (!Directory.Exists(tests))
			Directory.CreateDirectory(tests);

		let testDir = Path.InternalCombine(.. scope .(), tests, name);
		if (Directory.Exists(testDir))
			Bail!("Test already exists");

		Package test = scope .();
		Try!(test.Create(testDir, name, .Binary));
		Try!(test.Make());

		BeefSpace workspace = Scoped!(Try!(BeefSpace.FromPackage(path)));
		HashSet<String> testFolder;
		if (workspace.WorkspaceFolders.GetValue("Tests") case .Ok(let folder))
			testFolder = folder;
		else
			testFolder = workspace.WorkspaceFolders[new $"Tests"] = new .();

		if (!testFolder.ContainsAlt(name))
			testFolder.Add(new .(name));

		if (!workspace.Projects.ContainsKeyAlt(name))
			workspace.Projects[new .(name)] = new .() { Path = new .(testDir) };

		Try!(workspace.Save());
		return .Ok;
	}

	mixin Scoped(var value)
	{
		defer:mixin delete value;
		value
	}

	// Make a workspace.
	// 
	// @remarks Steps:
	// 	1. Update the registry if necessary.
	//
	// 	2. Resolve the dependency tree if lock doesn't
	// 	   already satisfy requirements.
	//
	// 	3. Fetch packages not found locally.
	//
	// 	4. Build the workspace file and package tree links.
	// 	   Goes through all packages included in the tree
	// 	   and creates references to dependencies and features.
	//
	// Depending on the value of 'GConsole.Quiet', this will display
	// a console interface with status, logs and progress bars.
	//
	/// Make a workspace.
	public Result<void> Make()
	{
		AssertOpen!();

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
		if (registry == null)
		{
			PathRegistry reg = new .("https://github.com/roguemacro/grill-index");
			registry = .Attach(reg);
			cache = new .((.)registry);
		}

		return ((PathRegistry)registry).Fetch();
	}

	Result<void> Resolve()
	{
		let previousLock = Lock;
		Resolver resolver = scope .(cache);
		Lock = Try!(resolver.Resolve(Manifest, path));
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

			let (_, fetched) = Try!(Packages.Install(pkg, version, cache, scope (stats) => {
				let percent = Math.Floor((float)stats.indexed_objects / stats.total_objects * 100);
				progress.UpdateText($"{pkg} {percent}%");
			})..Context($"Failed to install {pkg} v{version}"));

			progress.Text.Set("");
			progress.Tick();

			if (fetched)
				Log.Print(Styled("Fetched")..Bright()..Green(), "{} v{}", pkg, version);
		}

		progress.Finish();
		return .Ok;
	}

	public Result<void> Build()
	{
		AssertOpen!();
		let builder = scope WorkspaceBuilder(Manifest, path, Packages);
		return builder.Build();
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

	/// Run unit-tests
	public Result<void> UnitTests()
	{
		AssertOpen!();

		return .Err;
	}

	mixin AssertOpen()
	{
		if (!isOpen)
			Bail!("No workspace is open");
	}
}