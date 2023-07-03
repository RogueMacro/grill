using System;
using System.Collections;
using System.IO;
using Click;
using Grill.Console;

using static BuildTools.Git.Git;

namespace Grill;

class Packages
{
	typealias InstallCallback = delegate void(git_indexer_progress* stats);

	Dictionary<String, Package> packages = new .() ~ DeleteDictionaryAndKeysAndValues!(_);

	String cachePath ~ delete _;

	public this(StringView path)
	{
		cachePath = new .(path);

		for (let dir in Directory.EnumerateDirectories(path))
		{
			Package package = new .("", "", false);
			dir.GetFileName(package.Identifier);
			dir.GetFilePath(package.Path);
			packages.Add(new .(package.Identifier), package);
		}
	}

	public Result<Package> Install(String name, Version version, RegistryCache cache, InstallCallback callback = null)
	{
		String ident = scope $"{name}-{version}";
		if (packages.GetValue(ident) case .Ok(let package))
			return package;

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
			CLI.Context.Report($"No version {version} found for {name}");
			return .Err;
		}

		return InstallGit(package.url, rev, ident, callback);
	}

	// TODO: Support revision, and identifiers for git repos, not only registry identifiers
	public Result<Package> InstallGit(String url, String rev = null, String ident = null, InstallCallback callback = null)
	{
		String path = scope .();
		if (ident != null)
			Path.InternalCombine(path, cachePath, ident);
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
			CLI.Context.Report($"{error.Message}");
			CLI.Context.Report($"Git clone of {url} failed");
			return .Err;
		}

		Package package = new .(ident, path, true);
		packages.Add(new .(ident), package);
		return package;
	}

	public class Package
	{
		public String Identifier ~ delete _;
		public String Path ~ delete _;
		public bool JustInstalled;

		public String Revision ~ delete _;

		public this(StringView ident, StringView path, bool justInstalled, String rev = null)
		{
			Identifier = new .(ident);
			Path = new .(path);
			JustInstalled = justInstalled;

			Revision = rev == null ? null : new .(rev);
		}
	}
}