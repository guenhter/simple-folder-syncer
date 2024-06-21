# Simple Folder Syncer

Currently workin progress. Updates to the Docs will follow...

## Prerequisites


### MSI Prerequisites
To build a new MSI, the following tools are needed

* .Net SDK (https://learn.microsoft.com/en-us/dotnet/core/install/windows)
* Wix Tools (https://wixtoolset.org/docs/intro/#msbuild)

The tools can be installed with

```ps1
# Install .Net
winget install Microsoft.DotNet.SDK.8

# Install Wix
dotnet tool install --global wix
```


## Useful Commands

### Create a new MSI


```ps1
cargo wix
```


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
