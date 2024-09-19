@echo off

set "BatchFilePath='%programfiles%\Simple Folder Syncer\simple-folder-syncer.exe'"
set "TaskName=Simple Folder Syncer"

@REM https://learn.microsoft.com/de-de/windows/win32/taskschd/schtasks
schtasks /create /f /tn "%TaskName%" /tr "%BatchFilePath%" /sc hourly /mo 1
