@echo off

set "TaskName=Simple Folder Syncer"

schtasks /delete /tn "%TaskName%" /f

:: Ignore any errors because it is anyway too late
exit 0
