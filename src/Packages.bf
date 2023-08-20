using System;
using System.Collections;
using System.IO;
using SyncErr;
using Grill.Resolution;

using static BuildTools.Git.Git;

namespace Grill;

class Packages : IEnumerable<Package>
{
	typealias InstallCallback = delegate void(git_indexer_progress* stats);

	Dictionary<String, Package> packages = new .() ~ DeleteDictionaryAndKeysAndValues!(_);

	String cachePath ~ delete _;

	public this(StringView path)
	{
		cachePath = new .(path);

		for (let dir in Directory.EnumerateDirectories(path))
		{
			let ident = dir.GetFileName(.. new .());
			let pkgPath = dir.GetFilePath(.. scope .());
			packages.Add(ident, new .(null, pkgPath));
		}
	}

	public Dictionary<String, Package>.ValueEnumerator GetEnumerator()
	{
		return packages.Values;
	}

	public Result<(Package, bool fetched)> Install(String name, Version version, RegistryCache cache, InstallCallback callback = null)
	{
		Identifier ident = new .(name, version);
		defer ident.ReleaseRef();
		if (packages.GetValue(ident.Str) case .Ok(var package))
		{
			if (package.Identifier == null)
				package.Identifier = ident..AddRef();
			return (package, false);
		}

		let package = Try!(cache.GetPackage(name));
		String rev = null;
		for (let (v, metadata) in package.versions)
		{
			if (v == version)
			{
				rev = metadata.rev;
				break;
			}
		}

		if (rev == null)
		{
			Errors.Report($"No version {version} found for {name}");
			return .Err;
		}

		return InstallGit(package.url, rev, ident, callback);
	}

	// TODO: Support revision, and identifiers for git repos, not only registry identifiers
	public Result<(Package, bool fetched)> InstallGit(String url, String rev = null, Identifier ident = null, InstallCallback callback = null)
	{
		String path = scope .();
		if (ident != null)
			Path.InternalCombine(path, cachePath, ident.Str);
		else
			path.Set(Paths.Temporary);

		Directory.DelTree(path);
		Directory.CreateDirectory(path);

		git_remote_callbacks remoteCallbacks = .();
		git_remote_init_callbacks(&remoteCallbacks, 1);
		var callback;
		remoteCallbacks.payload = &callback;
		remoteCallbacks.transfer_progress = (stats, payload) => {
			InstallCallback* cb = (.)payload;
			(*cb)(stats);
			return 0;
		};
		

		git_fetch_options fetchOptions = .();
		git_fetch_options_init(&fetchOptions, 1);
		fetchOptions.callbacks = remoteCallbacks;

		git_clone_options cloneOptions = .();
		git_clone_options_init(&cloneOptions, 1);
		cloneOptions.fetch_opts = fetchOptions;

		git_repository* repo;
		if (git_clone(out repo, url, path, &cloneOptions) != .GIT_OK)
		{
			let error = BuildTools.GitTools.GetLastError();
			Errors.Report($"{error.Message}");
			Errors.Report($"Git clone of {url} failed");
			return .Err;
		}

		Package package = new .(ident, path);
		packages.Add(new .(ident.Str), package);
		return (package, true);
	}

	public class Package
	{
		public Identifier Identifier ~ if (_ != null) _.ReleaseRef();
		public String Path ~ delete _;

		public this(Identifier ident, StringView path)
		{
			Identifier = ident;
			if (ident != null)
				ident.AddRef();
			Path = System.IO.Path.GetFullPath(scope .(path), .. new .());
		}
	}
}