namespace System.IO;

extension Directory
{
	public static bool IsEmpty(StringView path)
	{
		var files = EnumerateFiles(path);
		var dirs = EnumerateDirectories(path);
		defer { files.Dispose(); dirs.Dispose(); }

		return (files.GetNext(), dirs.GetNext()) case (.Err, .Err);
	}
}