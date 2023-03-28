using System;
using System.Collections;
using System.IO;
using Serialize;
using Toml;
using static BuildTools.Git.Git;

namespace Grill
{
    class Program
    {
        public static int Main(String[] args)
        {
			git_libgit2_init();

			IndexManager.Update();
			let index = IndexManager.Index;

			git_libgit2_shutdown();
			Console.WriteLine("Done!");
			Console.Read().IgnoreError();
            return 0;
        }

		static mixin TryGit(var error)
		{
			if (error < 0) {
			  GitError e = BuildTools.GitTools.GetLastError();
			  System.Diagnostics.Debug.WriteLine("Error {}/{}: {}", error, e.ErrorClass, e.Message);
			  return -1;
			}
		}
    }
}
    