
$ErrorActionPreference = 'Stop';
$toolsDir = "$(Split-Path -parent $MyInvocation.MyCommand.Definition)"
$url = ''
$url64 = ''

$packageArgs = @{
  packageName    = $env:ChocolateyPackageName
  unzipLocation  = $toolsDir
  fileType       = 'MSI'
  url            = $url
  url64bit       = $url64
  file           = "$toolsDir/grill-$env:ChocolateyPackageVersion-x86_64.msi"

  softwareName   = 'grill*'

  checksum       = '63922845B733D041275F41CF286C27AE96575607E028FB40AFE974F150674FED'
  checksumType   = 'sha256'
  checksum64     = ''
  checksumType64 = 'sha256'

  silentArgs     = "/qn /norestart /l*v `"$($env:TEMP)\$($packageName).$($env:chocolateyPackageVersion).MsiInstall.log`""
  validExitCodes = @(0, 3010, 1641)
}

Install-ChocolateyInstallPackage @packageArgs



















