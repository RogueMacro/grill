using System;
using System.Collections;
using System.IO;
using Grill.Util;
using Iterators;
using SyncErr;
using Grill.Beef;
using Grill.Console;

namespace Grill;

class WorkspaceBuilder
{
	Manifest manifest;
	Packages installedPackages;

	String workspacePath ~ delete _;

	BeefSpace beefspace ~ delete _;
	HashSet<String> packageFolder;
	Dictionary<String, String> connects = new .() ~ DeleteDictionaryAndKeysAndValues!(_);

	public this(Manifest manifest, StringView path, Packages packages)
	{
		this.manifest = manifest;
		this.workspacePath = new .(path);
		this.installedPackages = packages;
	}

	/// Builds a workspace file and links all packages together.
	public Result<void> Build()
	{
		let beefprojPath = Path.InternalCombine(.. scope .(), workspacePath, "BeefProj.toml");
		BeefProj proj = File.Exists(beefprojPath) ? Try!(BeefProj.Read(beefprojPath)) : BeefProj.CreateDefault(beefprojPath);
		GetOrCreate!(proj.Project);
	    GetOrCreate!(proj.Project.Name).Set(manifest.Package.Name);
		Try!(proj.Save());
		delete proj;

		let beefspacePath = Path.InternalCombine(.. scope .(), workspacePath, "BeefSpace.toml");
		beefspace = File.Exists(beefspacePath) ? Try!(BeefSpace.Read(beefspacePath)) : BeefSpace.CreateDefault(beefspacePath);

		DeleteDictionaryAndKeysAndValues!(beefspace.Projects);
		beefspace.Projects = new .();

		ClearAndDeleteItems!(GetOrCreate!(beefspace.Locked));

		GetOrCreate!(beefspace.Workspace);
		var startupProject = GetOrCreate!(beefspace.Workspace.StartupProject);
		if (manifest.Workspace?.StartupProject != null)
			startupProject.Set(manifest.Workspace.StartupProject);
		else
			startupProject.Set(manifest.Package.Name);
		
		GetOrCreate!(beefspace.WorkspaceFolders);

		List<(String name, String path)> integrationTests = scope .();
		let testDir = Path.InternalCombine(.. scope .(), workspacePath, "tests");
		for (let dir in Directory.EnumerateDirectories(testDir))
		{
			let name = dir.GetFileName(.. new .());
			if (name == "common")
			{
				delete name;
				continue;
			}

			let path = dir.GetFilePath(.. new .());
			integrationTests.Add((name, path));
		}

		if (!integrationTests.IsEmpty)
		{
			if (!beefspace.WorkspaceFolders.ContainsKey("Tests"))
				beefspace.WorkspaceFolders[new .("Tests")] = new .();

			var tests = beefspace.WorkspaceFolders["Tests"];
			ClearAndDeleteItems!(tests);

			for (let (name, path) in integrationTests)
			{
				tests.Add(name);
				beefspace.Projects.Add(new .(name), new .() {
					Path = path
				});
			}
		}

		if (!beefspace.WorkspaceFolders.ContainsKey("Packages"))
			beefspace.WorkspaceFolders[new .("Packages")] = packageFolder = new .();
		else
			packageFolder = beefspace.WorkspaceFolders["Packages"];
		ClearAndDeleteItems!(packageFolder);

		Try!(Connect(manifest.Package.Name, manifest.Package.Version, workspacePath, false));

		if (manifest.Workspace?.Members != null)
		{
			for (let member in manifest.Workspace.Members)
			{
				let memberPath = Path.InternalCombine(.. scope .(), workspacePath, member);
				let memberManifest = Try!(Manifest.FromPackage(memberPath));
				defer delete memberManifest;
				Try!(Connect(memberManifest.Package.Name, memberManifest.Package.Version, memberPath, false));
			}
		}

		if (!beefspace.Projects.ContainsKey(startupProject))
			Bail!(scope $"Startup-project '{startupProject}' not found. Did you add it as a member?");

		return beefspace.Save();
	}

	/// Connect a package to the network of projects. This will find and
	/// create references to dependencies and features, recursively.
	Result<String> Connect(StringView pkgName, Version? version, String path, bool isPkg)
	{
		var path;
		path = Path.GetFullPath(path, .. scope .());
		if (connects.GetValue(path) case .Ok(let ident))
			return .Ok(ident);

		Manifest manifest = Try!(Manifest.FromPackage(path));
		BeefProj proj;
		if (!(BeefProj.FromPackage(path) case .Ok(out proj)))
			proj = BeefProj.CreateDefault(Path.InternalCombine(.. scope .(), path, "BeefProj.toml"));

		defer { delete manifest; delete proj; }

		if (manifest.Package.Name != pkgName)
			Bail!(scope $"Package '{pkgName}' not found in {path}. Did you mean '{manifest.Package.Name}'?");

		GetOrCreate!(proj.Project);
		if (isPkg)
			proj.Project.TargetType = .Library;
		let isBinary = proj.Project.TargetType == .Binary;

		ClearAndDeletePairs!(GetOrCreate!(proj.Dependencies));
		if (manifest.Package.Corlib)
			proj.Dependencies[new .("corlib")] = new .("*");
		
		String ident;
		if (isPkg && version != null)
			ident = new $"{pkgName}-{version}";
		else
			ident = new .(pkgName);

		connects.Add(new .(path), ident);

		GetOrCreate!(proj.Project.Name).Set(ident);

		if (proj.Project.ProcessorMacros != null)
			ClearAndDeleteItems!(proj.Project.ProcessorMacros);

		if (manifest.Dependencies != null)
		{
			DepLoop:
			for (let (name, dep) in manifest.Dependencies)
			{
				if (dep case .Local(let local))
				{
					var depPath = Path.InternalCombine(.. scope .(), path, local.Path);
					depPath = Path.GetFullPath(depPath, .. scope .());
					let depManifest = Try!(Manifest.FromPackage(depPath));
					defer delete depManifest;

					if (depManifest.Features?.Optional != null)
					{
						for (let feature in depManifest.Features.Optional.Values)
						{
							if (feature case .Project(let p))
							{
								var featurePath = Path.InternalCombine(.. scope .(), depPath, p);
								featurePath = Path.GetFullPath(featurePath, .. scope .());
								if (featurePath == path)
								{
									// We are a feature-project of this dependency.
									// As an "extension" of the dependency project,
									// we rely on it as a dependency.
									proj.Dependencies[new .(depPath)] = new .("*");
									continue DepLoop;
								}
							}
						}
					}
	
					String depIdent;
					if (isPkg)
					{
						// We are a package in someone else's workspace,
						// so we don't want our dependencies names to collide
						depIdent = Try!(Connect(
							scope $"{pkgName}/{name}",
							null,
							depPath,
							isPkg
						));
					}
					else if (!(depPath.StartsWith(path) || path.StartsWith(depPath)))
					{
						// Dependency is an external package outside our root package
						depIdent = Try!(Connect(
							name,
							null,
							depPath,
							isPkg
						));
					}
					else
					{
						// Dependency a local project in our workspace
						depIdent = Try!(Connect(
							name,
							null,
							depPath,
							// If we are a binary application then local
							// dependencies should not be considered " external packages"
							isPkg && !isBinary
						));
					}
	
					proj.Dependencies.Add(
						!isPkg || isBinary ? new .(name) : depIdent,
						new .("*")
					);
	
					let features = local.Features.GetEnumerator();
					if (local.DefaultFeatures && depManifest.Features.Default != null)
					{
						features.Chain(depManifest.Features.Default);
					}	
	
					var depProj = Try!(BeefProj.FromPackage(depPath));
					defer delete depProj;
					if (!connects.ContainsKey(depPath))
					{
						depProj.Project.ProcessorMacros.Clear();
						connects.Add(depPath, depIdent);
					}
	
					for (let feature in features)
					{
						if (depManifest.Features.Optional.GetValue(feature) case .Ok(.Project(let p)))
						{
							var featurePath = Path.InternalCombine(.. scope .(), depPath, p);
							featurePath = Path.GetFullPath(featurePath, .. scope .());
							if (featurePath == path)
								continue;
						}
	
						List<String> featureIdents = scope .();
						Try!(EnableFeature(depPath, feature, featureIdents));
	
						featureIdents
							.GetEnumerator()
							.Map(scope (i) => (key: i, value: new $"*"))
							.Collect(proj.Dependencies);
	
						let macro = new $"FEATURE_{scope String(feature)..ToUpper()}";
						if (!depProj.Project.ProcessorMacros.Add(macro))
							delete macro;
					}

					Try!(depProj.Save());
					continue;
				}
	
				for (let package in installedPackages)
				{
					if (package.Identifier.Name == name)
					{
						IEnumerator<String> features = null;
						bool enableDefaultFeatures = true;
						bool add;
						switch ((package.Identifier.Version, dep))
						{
						case (.SemVer(let v), .Simple(let req)):
							add = req.Matches(v);
							break;
						case (.SemVer(let v), .Advanced(let advanced)):
							features = advanced.Features.GetEnumerator();
							enableDefaultFeatures = advanced.DefaultFeatures;
							add = advanced.Req.Matches(v);
							break;
						case (.Git(let rev), .Git(let git)):
							add = rev == git.Rev;
							break;
						default:
							add = false;
						}
	
						if (add)
						{
							var ident;
							var version;
	
							version = null;
							if (package.Identifier.Version case .SemVer(let v))
								version = v;
	
							ident = Try!(Connect(package.Identifier.Name, version, package.Path, true));
							proj.Dependencies.Add(new .(ident), new .("*"));
	
							let depManifest = Try!(Manifest.FromPackage(package.Path));
							defer delete depManifest;
							let weAreFeature = depManifest.Features?.Optional == null ? false : depManifest.Features.Optional.Values.Any(scope (feature) => {
								if (feature case .Project(var featurePath))
								{
									featurePath = Path.InternalCombine(.. scope .(), package.Path, featurePath);
									featurePath = Path.GetFullPath(featurePath, .. scope .());
									return featurePath == package.Path;
								}
	
								return false;
							});
	
							if (weAreFeature)
								continue DepLoop;
	
							if (features != null)
							{
								if (enableDefaultFeatures && depManifest.Features.Default != null)
									features = features.Chain(depManifest.Features.Default);
	
								let depProj = Try!(BeefProj.FromPackage(package.Path));
								defer delete depProj;
								for (let feature in features)
								{
									Log.Info($"Enabling feature {feature} of {name}");
									List<String> featureIdents = scope .();
									Try!(EnableFeature(package.Path, feature, featureIdents));
	
									featureIdents
										.GetEnumerator()
										.Map(scope (i) => (key: i, value: new $"*"))
										.Collect(proj.Dependencies);
	
									depProj.Project.ProcessorMacros.Add(new $"FEATURE_{scope String(feature)..ToUpper()}");
								}
	
								Try!(depProj.Save());
							}
							
							continue DepLoop;
						}
					}
				}
	
				Log.Error($"Could not find dependency {name} of {pkgName}");
			}
		}

		beefspace.Projects.Add((
			new .(isPkg ? ident : pkgName),
			new .() {
				Path = new .(path)
			}
		));

		if (isPkg)
		{
			beefspace.Locked.Add(new .(ident));
			packageFolder.Add(new .(ident));
		}

		Try!(proj.Save());

		return .Ok(ident);
	}

	/// Will "prime" a feature of the package at the given path by
	/// connecting it to it's dependencies or enabling sub-features.
	/// Enabled features will be added to idents.
	Result<void> EnableFeature(String path, String feature, List<String> idents)
	{
		let manifest = Try!(Manifest.FromPackage(path));
		if (!manifest.Features.Optional.ContainsKey(feature))
			Bail!(scope $"Unknown feature '{feature}' for '{manifest.Package.Name}'");

		switch (manifest.Features.Optional.GetValue(feature).Get())
		{
		case .List(let subFeatures):
			for (let sub in subFeatures)
				Try!(EnableFeature(path, sub, idents));
		case .Project(let featurePath):
			let ident = new $"{manifest.Package.Name}-{manifest.Package.Version}/{feature}";
			let fullPath = Path.InternalCombine(.. scope .(), path, featurePath);
			if (Connect(ident, null, fullPath, true) case .Err)
			{
				delete ident;
				return .Err;
			}

			idents.Add(ident);
		}

		return .Ok;
	}

	mixin GetOrCreate(var val)
	{
		if (val == null)
			val = new .();
		val
	}
}