@echo off

set "BatchFilePath='%programfiles%\Simple Folder Syncer\simple-folder-syncer.exe'"
set "ReconnectDrivePath='%programfiles%\Simple Folder Syncer\reconnect-netdrive.bat'"

set "TaskName=Simple Folder Syncer"
set "TaskDescription=Run my batch file daily"

@REM https://learn.microsoft.com/de-de/windows/win32/taskschd/schtasks
schtasks /create /f /tn "%TaskName%" /tr "%BatchFilePath%" /sc hourly /mo 1

@REM This is only a workaround until the actual SSHFS drive reconnecting is implemented.
schtasks /create /f /tn "Reconnect SSHFS drive" /tr "%ReconnectDrivePath%" /sc minute /mo 30
