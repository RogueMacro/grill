using System;
using System.IO;
using Toml;

using static BuildTools.Git.Git;

namespace Grill;

/// A registry that stores packages as files in a filesystem.
/// A package might be found at registry/my-package.toml,
/// or for a larger ecosystem might be split into separate directories;
/// registry/my/-p/my-package.toml.
///
/// This registry is hosted as a remote git repository, and
/// is fetched to update the local registry.
class PathRegistry : IRegistry
{
	String url ~ delete _;

	readonly String hash = new .() ~ delete _;
	readonly String path = new .() ~ delete _;

	public this(StringView registryUrl)
	{
		url = new .(registryUrl);
		scope String(registryUrl)
			..ToUpper()
			.GetHashCode()
			.ToString(hash);

		Path.InternalCombine(path, Paths.Home, "registry", hash);
	}

	public Result<PackageMetadata> GetPackage(StringView name)
	{
		String packagePath = Path.InternalCombine(.. scope .(), path, name);
		if (!File.Exists(packagePath))
			return .Err;

		String file = File.ReadAllText(packagePath, .. scope .());
		return Toml.Deserialize<PackageMetadata>(file);
	}

	/// Fetch new packages and versions from the remote registry.
	/// Will override any changes made to the local registry.
	public Result<void> Fetch()
	{
		if (!Directory.Exists(path))
		{
			git_repository* repo = null;
			if (git_clone(out repo, url.CStr(), path.CStr(), null) != .GIT_OK)
				return .Err;
		}
		else
		{
			if (GitPull() case .Err(let err))
				return .Err;
		}

		return .Ok;
	}

	Result<void, GitErrorCode> GitPull()
	{
		git_repository* repo = null;
		git_remote* remote = null;
		GitCheckError!(git_repository_open((.)&repo, path.CStr()));

		GitCheckError!(git_remote_lookup(&remote, repo, "origin"));
		GitCheckError!(git_remote_fetch(remote, null, null, null));

		git_oid merge_oid;
		GitCheckError!(git_repository_fetchhead_foreach(repo, (ref_name, remote_url, oid, is_merge, payload) => {
			if (is_merge > 0)
				git_oid_cpy((git_oid*)payload, oid);
			return 0;
		}, &merge_oid));

		git_annotated_commit* head = ?;
		GitCheckError!(git_annotated_commit_lookup(&head, repo, &merge_oid));

		GitCheckError!(git_reset_from_annotated(repo, head, .GIT_RESET_HARD, null));

		GitCheckError!(git_annotated_commit_free(head));
		git_remote_free(remote);
		git_repository_free(repo);

		return .Ok;
	}

	mixin GitCheckError(GitErrorCode val)
	{
		if (val != .GIT_OK)
			return .Err(val);
	}
}