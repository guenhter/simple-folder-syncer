# Simple Folder Syncer


## Prerequisites


### MSI Prerequisites
To build a new MSI, the following tools are needed

* .Net SDK (https://learn.microsoft.com/en-us/dotnet/core/install/windows)
* Wix Cargo Plugin
* Wix Tools (https://wixtoolset.org/docs/intro/#msbuild)

The tools can be installed with

```ps1
# Install .Net
winget install Microsoft.DotNet.SDK.8

# Install Wix
dotnet tool install --global wix
```



## Create a new MSI

```ps1
cargo wix
```


```ps1
# Install an MSI and create the log for the installation process
msiexec /i my.msi /l*v install.log

# Unsinstall the MSI
msiexec /x my.msi /l*v uninstall.log
```
