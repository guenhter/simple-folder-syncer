@echo off

set "TaskName=Simple Folder Syncer"

schtasks /delete /tn "%TaskName%" /f
