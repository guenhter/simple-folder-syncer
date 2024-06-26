# Simple Folder Syncer

The Simple Folder Syncer is a thin wrapper around the Windows program "Robocopy". Robocopy is used
to mirror one folder to another. The reason for this wrapper is to make the exclude handling
a little bit easier and narrow the program down to only syncing folders.


## Prerequisites

* Robocopy (should acutally be available on every Windows machine)


## Configuration

The configuration of the Simple Folder Syncer is done via a configuration file exclusively. No CLI parameters are supported yet.

The configuratoin file must be located under `%HOME%/folder_sync_config.yaml`

The content of the file looks like this:

```yaml
---
source: C:\Users\foo
target: Z:\
create_last_sync_result_file: true
exclude_root_source_hidden_entries: true
exclude_paths:
  - C:\Users\foo\Downloads
  - C:\Users\foo\OneDrive
```

| Field    | Description |
| -------- | ------- |
| source  | The directory which should be mirrored to the target. Only the content of the source is mirrored, not the directory iteslf.    |
| target | The target directory where the content of the source directory should be mirrored to. |
| exclude_root_source_hidden_entries    | If set to true, hidden entries in the root of the source directory are excluded from the sync. This does not affect hidden entries in any subdirectory of the source. |
| create_last_sync_result_file    | If set to true, a file named `last_sync_result.txt` is created in the target directory. The file contains a summary of the last sync run |
| exclude_paths    | A list of additional paths to be excluded from the sync. Currently, no support for wildcards is provided. Excluded paths will get deleted in the target. |


## Limitations

* Currently only Windows is supported


## Contribution

Contribution are always welcome in any form.

You acknowledge and agree that the owner reserve the right to change the license of the Work, including but not limited to all Contributions previously submitted by You, at any time without the need for approval from You or any other contributor.

## License

This project is licensed under the [MIT license].

[MIT license]: https://github.com/guenhter/simple-folder-syncer/blob/main/LICENSE
