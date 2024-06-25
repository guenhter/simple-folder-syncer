# Simple Folder Syncer

The Simple Folder Syncer is a thin wrapper around the Windows program "Robocopy". Robocopy is used
to mirror one folder to another. The reason for this wrapper is to make the exclude handling
a little bit easier and narrow the program down to only syncing folders.


## Limitations

* Currently only Windows is supported


## Prerequisites

* Robocopy (should acutally be available on every Windows machine)



## Development Notes

A MSI file is produced to make it easier installable.

### MSI Prerequisites

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

### Create a new MSI

```ps1
# Building the MSI
wix build .\package.wxs -o my.msi -arch x64

# Install an MSI and create the log for the installation process
msiexec /i my.msi /l*v install.log

# Unsinstall the MSI
msiexec /x my.msi /l*v uninstall.log
```


## Contribution

Contribution are always welcome in any form.

You acknowledge and agree that the owner reserve the right to change the license of the Work, including but not limited to all Contributions previously submitted by You, at any time without the need for approval from You or any other contributor.

## License

This project is licensed under the [MIT license].

[MIT license]: https://github.com/guenhter/simple-folder-syncer/blob/main/LICENSE
