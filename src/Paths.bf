using System;
using System.Collections;
using System.IO;
using System.Text;

namespace Grill;

static class Paths
{
	public const String MANIFEST_FILENAME = "Package.toml";
	public const String LOCK_FILENAME = "Package.lock";
	public const String PACKAGE_DIRECTORY = "pkg";
	public const String TEST_DIRECTORY = "tests";

	public static String Home ~ delete _;
	public static String Temporary ~ delete _;
	public static String BeefLibs ~ delete _;
	public readonly static String BeefPath ~ delete _;

	public static this()
	{
		let vars = Environment.GetEnvironmentVariables(.. new .());
		defer { DeleteDictionaryAndKeysAndValues!(vars); }
		if (!vars.ContainsKey("BeefPath"))
			Runtime.FatalError("No Beef installation found");
		BeefPath = new .(vars["BeefPath"]);

		Home = GetProfileDirectory(.. new .());
		Path.InternalCombine(Home, ".grill");
		Temporary = Path.InternalCombine(.. new .(), Home, "tmp");

		BeefLibs = Path.InternalCombine(.. new .(), BeefPath, "BeefLibs");
	}

	public static void ClearTemporary()
	{
		if (Directory.Exists(Temporary))
			Directory.DelTree(Temporary);
		Directory.CreateDirectory(Temporary);
	}

	public static void BeefLib(StringView lib, String buffer)
	{
		Path.InternalCombine(buffer, BeefLibs, lib);
	}

	static Result<void> GetProfileDirectory(String buffer)
	{
		char16* path = null;
		int result = SHGetKnownFolderPath(FOLDERID_Profile, 0, Windows.Handle.NullHandle, &path);
		if (result != 0 || path == null)
			return .Err;

		UTF16.Decode(path, buffer);
		return .Ok;
	}

	[Import("shell32.lib"), CLink, CallingConvention(.Stdcall)]
	static extern int SHGetKnownFolderPath(GUID rfid, uint32 dwFlags, Windows.Handle hToken, char16** ppszPath);

	[CRepr]
	struct GUID {
		uint32 Data1;
		uint16 Data2;
		uint16 Data3;
		uint8[8] Data4;

		public this(uint32 l, uint16 w1, uint16 w2, uint8[8] b)
		{
			Data1 = l;
			Data2 = w1;
			Data3 = w2;
			Data4 = b;
		}
	}

	const GUID FOLDERID_Profile = GUID(0x5E6C858F, 0x0E22, 0x4760, uint8[](0x9A, 0xFE, 0xEA, 0x33, 0x17, 0xB6, 0x71, 0x73));
}