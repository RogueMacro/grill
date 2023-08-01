using System;
using System.Collections;
using System.IO;
using Grill.Util;
using Iterators;
using SyncErr;
using Grill.Console;

namespace Grill;

class WorkspaceBuilder
{
	String packageName ~ delete _;
	Version packageVersion;
	String workspacePath ~ delete _;

	BeefSpace beefspace ~ delete _;
	HashSet<String> packageFolder;
	Dictionary<String, String> connects = new .() ~ DeleteDictionaryAndKeysAndValues!(_);
	Packages packages;

	public this(Manifest manifest, StringView path, Packages packages)
	{
		this.packageName = new .(manifest.Package.Name);
		this.packageVersion = manifest.Package.Version;
		this.workspacePath = new .(path);
		this.packages = packages;
	}

	public Result<void> Build()
	{
		let beefprojPath = Path.InternalCombine(.. scope .(), workspacePath, "BeefProj.toml");
		BeefProj proj = File.Exists(beefprojPath) ? Try!(BeefProj.Read(beefprojPath)) : BeefProj.CreateDefault(beefprojPath);
		GetOrCreate!(proj.Project);
	    GetOrCreate!(proj.Project.Name).Set(packageName);
		Try!(proj.Save());
		delete proj;

		let beefspacePath = Path.InternalCombine(.. scope .(), workspacePath, "BeefSpace.toml");
		beefspace = File.Exists(beefspacePath) ? Try!(BeefSpace.Read(beefspacePath)) : BeefSpace.CreateDefault(beefspacePath);

		GetOrCreate!(beefspace.Workspace);
		GetOrCreate!(beefspace.Workspace.StartupProject).Set(packageName);

		DeleteDictionaryAndKeysAndValues!(beefspace.Projects);
		beefspace.Projects = new .();

		beefspace.Projects[new .("corlib")] = new .() {
			Path = Paths.BeefLib("corlib", .. new .())
		};

		ClearAndDeleteItems!(GetOrCreate!(beefspace.Locked));
		beefspace.Locked.Add(new .("corlib"));

		GetOrCreate!(beefspace.WorkspaceFolders);
		if (!beefspace.WorkspaceFolders.ContainsKey("Packages"))
			beefspace.WorkspaceFolders[new .("Packages")] = packageFolder = new .();
		else
			packageFolder = beefspace.WorkspaceFolders["Packages"];

		ClearAndDeleteItems!(packageFolder);

		Try!(Connect(packageName, packageVersion, workspacePath, false));

		return beefspace.Save();
	}

	mixin GetOrCreate(var val)
	{
		if (val == null)
			val = new .();
		val
	}

	Result<String> Connect(StringView pkgName, Version? version, String path, bool isPkg)
	{
		var path;
		path = Path.GetFullPath(path, .. scope .());
		if (connects.GetValue(path) case .Ok(let ident))
			return .Ok(ident);

		Manifest manifest = Try!(Manifest.FromPackage(path));
		BeefProj proj;
		//if (!(BeefProj.FromPackage(path) case .Ok(out proj)))
		//	proj = BeefProj.CreateDefault(Path.InternalCombine(.. scope .(), path, "BeefProj.toml"));
		proj = Try!(BeefProj.FromPackage(path));

		defer { delete manifest; delete proj; }

		let isBinary = proj.Project.TargetType == .BeefConsoleApplication;

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
	
					for (let feature in depManifest.Features.Values)
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
	
					String depIdent;
					if (path.StartsWith(depPath))
					{
						// We are a subpackage to this dependency
						depIdent = Try!(Connect(
							scope $"{pkgName}/{name}",
							null,
							depPath,
							isPkg
						));
					}
					else if (depPath.StartsWith(path))
					{
						// Dependency is a subpackage to us
						depIdent = Try!(Connect(
							name,
							null,
							depPath,
							// If we are a binary application then local
							// dependencies should not be considered packages 
							isPkg && !isBinary
						));
					}
					else
					{
						// Dependency is an external package outside our root package
						depIdent = Try!(Connect(
							name,
							null,
							depPath,
							isPkg
						));
					}
	
					proj.Dependencies.Add(
						!isPkg || isBinary ? new .(name) : depIdent,
						new .("*")
					);
	
					let features = local.Features.GetEnumerator();
					if (local.DefaultFeatures && depManifest.Features.ContainsKey("Default"))
					{
						let defaults = depManifest.Features["Default"];
						if (defaults case .List(let defaultDepFeatures))
							features.Chain(defaultDepFeatures.GetEnumerator());
						else
							Bail!(scope $"Default features of '{name}' must be a list of features, not a project path");
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
						if (depManifest.Features.GetValue(feature) case .Ok(.Project(let p)))
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
	
				for (let package in packages)
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
							let weAreFeature = depManifest.Features == null ? false : depManifest.Features.Values.Any(scope (feature) => {
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
								if (enableDefaultFeatures && depManifest.Features.GetValue("Default") case .Ok(.List(let defaultFeatures)))
									features = features.Chain(defaultFeatures.GetEnumerator());
	
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
	
				Log.Print("[Error]", $"{pkgName} missing dependency {name}");
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

	Result<void> EnableFeature(String path, String feature, List<String> idents)
	{
		let manifest = Try!(Manifest.FromPackage(path));
		if (!manifest.Features.ContainsKey(feature))
			Bail!(scope $"Unknown feature '{feature}' for '{manifest.Package.Name}'");

		switch (manifest.Features.GetValue(feature).Get())
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
}