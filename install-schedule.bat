@echo off

set "BatchFilePath='%programfiles%\Simple Folder Syncer\simple-folder-syncer.exe'"

set "TaskName=Simple Folder Syncer"
set "TaskDescription=Run my batch file daily"

:: https://learn.microsoft.com/de-de/windows/win32/taskschd/schtasks
schtasks /create /f /tn "%TaskName%" /tr "%BatchFilePath%" /sc hourly
