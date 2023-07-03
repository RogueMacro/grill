using System;
using System.Collections;

namespace Grill.Resolver;

class Resolver
{
	RefCounted<RegistryCache> cache ~ _.Release();

	public this(RefCounted<RegistryCache> cache)
	{
		this.cache = cache..AddRef();
	}

	public Result<Lock> Resolve(Manifest manifest)
	{
		// Add the root dependencies
		List<Candidate> candidates = scope .();
		for (let (name, dependency) in manifest.Dependencies)
		{
			switch (dependency)
			{
			case .Simple(let req):
				candidates.Add(scope:: .(name, req, cache));
			}
		}

		// Remember how many root candidates, since
		// we don't want to remove these.
		let rootCandidatesCount = candidates.Count;

		int i = 0;
		while (true)
		{
			while (i >= 0 && i < candidates.Count)
			{
				Candidate candidate = candidates[i];
				if (!cache->ContainsPackage(candidate.Name))
					return .Err;

				Result<Version> nextVersion = .Err;
				VersionLoop:
				while (candidate.NextVersion() case .Ok(let version))
				{
					// Check for conflicts with already selected candidates.
					for (let c in candidates)
					{
						if (!c.Version.HasValue || c == candidate)
							continue;

						// Two versions are conflicting if they have the same major version,
						// but a different minor/patch versions. I.e. '1.2.3' is conflicting
						// with '1.3.3'.
						// Different major versions for the same dependency is allowed.
						// This is for convenience when using libraries together, as
						// different versions can't be mixed.
						let v = c.Version.Value;
						if (c.Name == candidate.Name &&
							v.Major == version.Major &&
							v != version)
						{
							// This version is a conflict.
							// Check the next version.
							continue VersionLoop;
						}
					}

					// This version matches the requirements and does not
					// come into conflict with any previously selected versions.
					nextVersion = version;
					break;
				}

				if (nextVersion case .Ok(let version))
				{
					let dependencies = cache->GetPackage(candidate.Name).Value.versions[version].deps;
					for (let (dep, req) in dependencies)
						candidates.Add(scope:: .(dep, req, cache));

					// "Commit" this version to the candidate and
					// move on to the next candidate.
					candidate.Version = version;
					i += 1;
				}
				else
				{
					if (i >= rootCandidatesCount)
					{
						// There were conflicts while selecting a version for this dependency.
						// Therefore we must remove all candidates added by this dependency.
						candidates.RemoveRange(candidates.Count - i, i);
					}
					else if (i == 0)
					{
						// There was no more versions left for the first
						// dependency that satisfies the requirements.
						// That means all combinations have been tried
						// and this manifest can't be resolved.
						return .Err;
					}
					else
					{
						// We can't remove a root dependency so we just
						// unset the version to make sure it is ready for
						// picking another version.
						candidate.Version = null;
						candidate.UpdateAvailableVersions(cache);
					}

					// Backtrack to the previous candidate
					// to try another combination.
					i -= 1;
				}
			}

			// When conflicts arise and we remove the invalidated candidates,
			// some of the previous dependencies might not get re-iterated so
			// that their dependencies are added as candidates again.
			// We go over the candidate list again to make sure all dependencies
			// are present and if not, we add the missing dependencies and restart
			// resolution loop.
			bool missingDependencies = false;
			for (let candidate in candidates)
			{
				let dependencies =
					cache->GetPackage(candidate.Name).Value
					.versions[candidate.Version.Value]
					.deps;

				for (let (dep, req) in dependencies)
				{
					if (candidates.FindIndex(scope (c) =>
							c.Name == dep &&
							req.Matches(c.Version.Value)
						) == -1)
					{
						// This dependency is missing a valid candidate.
						candidates.Add(scope:: .(dep, req, cache));
					}
				}
			}

			if (!missingDependencies)
			{
				// All dependencies have a valid candidate so the
				// resolution is ready, and we stop the resolution loop.
				break;
			}
			
			// We need to resolve the missing dependencies.
			// The resolution loop will continue automatically.
		}

		Lock lock = new .();
		for (let candidate in candidates)
		{
			HashSet<Version> versions;
			if (lock.GetValue(candidate.Name) case .Ok(let v))
				versions = v;
			else
				versions = lock.Add(new .(candidate.Name), .. new .());
			versions.Add(candidate.Version.Value);
		}

		return .Ok(lock);
	}

	class Candidate
	{
		public readonly String Name ~ delete _;
		public readonly VersionReq Requirement;
		public Version? Version;
		public List<Version> AvailableVersions ~ delete _;

		public this(StringView name, VersionReq req, RegistryCache cache)
		{
			Name = new .(name);
			Requirement = req;
			AvailableVersions = new .();
			UpdateAvailableVersions(cache).IgnoreError();
		}

		/// Updates AvailableVersions to all that matches the
		/// version requirement.
		public Result<void> UpdateAvailableVersions(RegistryCache cache)
		{
			AvailableVersions.Clear();
			let package = Try!(cache.GetPackage(Name));
			for (let version in package.versions.Keys)
			{
				if (Requirement.Matches(version))
					AvailableVersions.Add(version);
			}
			AvailableVersions.Sort();
			return .Ok;
		}

		/// Returns the next latest available version.
		public Result<Version> NextVersion()
		{
			if (AvailableVersions.IsEmpty)
				return .Err;
			return AvailableVersions.PopBack();
		}
	}
}