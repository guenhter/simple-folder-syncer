# MSI

## MSI Prerequisites

This project uses the Windows Installer XML (WiX) Toolset for building MSI packages.
The following tools are needed:

* .Net SDK (https://learn.microsoft.com/en-us/dotnet/core/install/windows)
* WiX Tools (https://wixtoolset.org/docs/intro/#msbuild)

The tools can be installed with

```powershell
# Install .Net
winget install Microsoft.DotNet.SDK.8

# Install WiX
dotnet tool install --global wix
```

Please note, that the cardo-wix plugin is not used because it is still besed
on WiX 3. The used WiX version of this project is the latest WiX 5.

## Create a new MSI

```ps1
# Building the MSI
wix build .\package.wxs -o my.msi -arch x64

# Install an MSI and create the log for the installation process
msiexec /i my.msi /l*v install.log

# Unsinstall the MSI
msiexec /x my.msi /l*v uninstall.log
```

