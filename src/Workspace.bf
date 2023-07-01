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

namespace Grill;

class Workspace
{
	public Manifest Manifest ~ delete _;

	String path ~ delete _;

	RefCounted<IRegistry> registry ~ _.Release();

	public this(StringView path)
	{
		this.path = new .(path);
	}

	public Result<void> Open()
	{
		String filePath = Path.InternalCombine(.. scope .(), path, Paths.MANIFEST_FILENAME);
		if (!File.Exists(filePath))
		{
			CLI.Context.Report("Manifest not found");
			return .Err;
		}

		String file = scope .();
		if (File.ReadAllText(filePath, file) case .Err)
		{
			CLI.Context.Report("Failed to read manifest");
			return .Err;
		}

		Serialize<Toml> serializer = scope .();
		switch (serializer.Deserialize<Manifest>(file))
		{
		case .Ok(let manifest):
			Manifest = manifest;
			return .Ok;
		case .Err:
			CLI.Context.Report($"Failed to deserialize manifest: {serializer.Error}");
			return .Err;
		}
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

		GConsole.WriteLine("             🍝 Enjoy your spaghetti!");

		return .Ok;
	}

	Result<void> Update()
	{
		PathRegistry reg = new .("https://github.com/roguemacro/grill-index");
		registry = .Attach(reg);
		reg.Fetch();

		return .Ok;
	}

	Result<void> Resolve()
	{
		Resolver resolver = scope .(registry..AddRef());
		let lock = Try!(resolver.Resolve(Manifest));
		defer delete lock;

		return .Ok;
	}

	Result<void> Fetch(MultiProgress multi)
	{
		String[] pkgs = scope .[]("Serialize", "Toml", "BuildTools", "Click");

		ProgressBar progress = scope .(pkgs.Count);
		multi.Add(progress);

		progress.Tick();
		for (let pkg in pkgs)
		{
			progress.UpdateText(pkg);

			Thread.Sleep(1000);

			progress.Text.Set("");
			progress.Tick();

			Log.Print("Fetched", .Green, pkg);
		}

		progress.Finish();
		multi.Remove(progress);

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