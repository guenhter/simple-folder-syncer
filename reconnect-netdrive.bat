@echo off

setlocal enabledelayedexpansion

for /F "tokens=*" %%l in ('net use ^| findstr /i "\\sshfs"') do set net_use_line=%%l

for %%a in (%net_use_line%) do (
    echo %%a|find "\\sshfs" >nul

    if errorlevel 1 (
        set previous=%%a
    ) else (
        set drive=!previous!
    )
)

IF [!drive!] == [] (
    exit /B 0
)

for %%a in (%net_use_line%) do (
    if "%%a"=="%drive%" (
        set "nextToken=1"
    ) else if defined nextToken (
        set "remote=%%a"
        set "nextToken="
    )
)

IF [!remote!] == [] (
    exit /B 0
)

echo Drive: !drive!
echo Remote: !remote!


if not exist "!drive!" (
    echo Reconnect drive now...
    net use !drive! !remote!
) else (
    echo Nothing to do. Drive is already connected.
)

endlocal

exit /B 0
