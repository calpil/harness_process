@echo off
rem ===========================================================================
rem  Harness Process - instalador para cmd.exe (Windows).
rem
rem  NO es un tercer instalador: es la puerta de entrada de cmd.exe al mismo
rem  setup_harness.ps1. Una tercera implementacion garantizaba drift con las
rem  otras dos, que es justo lo que tests/parity_check.sh existe para evitar.
rem  Aca solo se resuelve lo que cmd.exe no sabe hacer solo: encontrar
rem  PowerShell, saltear la ExecutionPolicy que rechaza un .ps1 sin firmar, y
rem  devolver el exit code de verdad.
rem
rem  Uso:  setup_harness.cmd [-DryRun] [-Reset] [-NoSubagents] [...]
rem        setup_harness.cmd --dry-run   (las opciones estilo .sh se traducen)
rem
rem  Los mensajes de error se emiten con GOTO y no dentro de bloques if(...):
rem  un parentesis sin escapar adentro de un bloque cierra el bloque antes de
rem  tiempo, y una ruta como "C:\Program Files (x86)\..." rompia el script en
rem  el peor momento, que es cuando algo ya habia fallado.
rem ===========================================================================
setlocal enabledelayedexpansion

set "HARNESS_SRC=%~dp0"
set "HARNESS_PS1=%HARNESS_SRC%setup_harness.ps1"

if not exist "%HARNESS_PS1%" goto sin_ps1

rem PowerShell 7 si esta; si no, el Windows PowerShell 5.1 que trae el sistema
rem (el .ps1 declara #requires -Version 5.1, asi que los dos sirven).
set "HARNESS_PWSH="
for %%P in (pwsh.exe) do if not defined HARNESS_PWSH set "HARNESS_PWSH=%%~$PATH:P"
if defined HARNESS_PWSH goto tiene_pwsh
for %%P in (powershell.exe) do if not defined HARNESS_PWSH set "HARNESS_PWSH=%%~$PATH:P"
if defined HARNESS_PWSH goto tiene_pwsh
if exist "%SystemRoot%\System32\WindowsPowerShell\v1.0\powershell.exe" set "HARNESS_PWSH=%SystemRoot%\System32\WindowsPowerShell\v1.0\powershell.exe"
if not defined HARNESS_PWSH goto sin_pwsh
:tiene_pwsh

rem Traduccion de las opciones estilo .sh a las del .ps1: --dry-run -> -DryRun.
rem Quien viene de la documentacion de Unix escribe --dry-run y merece que
rem funcione, en vez de un error de parametro. Lo demas pasa tal cual.
set "HARNESS_ARGS="
:traducir
if "%~1"=="" goto lanzar
set "HARNESS_ARG=%~1"
if not "!HARNESS_ARG:~0,2!"=="--" goto sumar_arg
call :a_pascal "!HARNESS_ARG:~2!"
set "HARNESS_ARG=-!HARNESS_PASCAL!"
:sumar_arg
set "HARNESS_ARGS=!HARNESS_ARGS! !HARNESS_ARG!"
shift
goto traducir

:lanzar
rem -ExecutionPolicy Bypass: el .ps1 recien clonado no esta firmado y la politica
rem por defecto de Windows lo rechaza. Es del alcance de ESTE proceso: no toca la
rem configuracion de la maquina, que es del usuario.
"%HARNESS_PWSH%" -NoProfile -ExecutionPolicy Bypass -File "%HARNESS_PS1%"%HARNESS_ARGS%
set "HARNESS_RC=%ERRORLEVEL%"
endlocal & exit /b %HARNESS_RC%

:a_pascal
rem --no-subagents -> NoSubagents: cada segmento entre guiones va con mayuscula.
set "HARNESS_PASCAL="
set "HARNESS_RESTO=%~1"
:a_pascal_loop
rem Las dos asignaciones pertenecen al mismo `for`: sin el bloque, cmd.exe
rem ejecuta la segunda fuera del cuerpo y deja HARNESS_RESTO sin avanzar.
for /f "tokens=1* delims=-" %%a in ("!HARNESS_RESTO!") do (
set "HARNESS_SEG=%%a"
set "HARNESS_RESTO=%%b"
)
if not defined HARNESS_SEG goto a_pascal_sigue
set "HARNESS_INICIAL=!HARNESS_SEG:~0,1!"
set "HARNESS_COLA=!HARNESS_SEG:~1!"
for %%L in (a b c d e f g h i j k l m n o p q r s t u v w x y z) do if "!HARNESS_INICIAL!"=="%%L" call :mayuscula %%L
set "HARNESS_PASCAL=!HARNESS_PASCAL!!HARNESS_INICIAL!!HARNESS_COLA!"
:a_pascal_sigue
set "HARNESS_SEG="
if defined HARNESS_RESTO goto a_pascal_loop
goto :eof

:mayuscula
rem cmd.exe no tiene upper(): la tabla es explicita, de una sola letra.
if "%~1"=="a" set "HARNESS_INICIAL=A"
if "%~1"=="b" set "HARNESS_INICIAL=B"
if "%~1"=="c" set "HARNESS_INICIAL=C"
if "%~1"=="d" set "HARNESS_INICIAL=D"
if "%~1"=="e" set "HARNESS_INICIAL=E"
if "%~1"=="f" set "HARNESS_INICIAL=F"
if "%~1"=="g" set "HARNESS_INICIAL=G"
if "%~1"=="h" set "HARNESS_INICIAL=H"
if "%~1"=="i" set "HARNESS_INICIAL=I"
if "%~1"=="j" set "HARNESS_INICIAL=J"
if "%~1"=="k" set "HARNESS_INICIAL=K"
if "%~1"=="l" set "HARNESS_INICIAL=L"
if "%~1"=="m" set "HARNESS_INICIAL=M"
if "%~1"=="n" set "HARNESS_INICIAL=N"
if "%~1"=="o" set "HARNESS_INICIAL=O"
if "%~1"=="p" set "HARNESS_INICIAL=P"
if "%~1"=="q" set "HARNESS_INICIAL=Q"
if "%~1"=="r" set "HARNESS_INICIAL=R"
if "%~1"=="s" set "HARNESS_INICIAL=S"
if "%~1"=="t" set "HARNESS_INICIAL=T"
if "%~1"=="u" set "HARNESS_INICIAL=U"
if "%~1"=="v" set "HARNESS_INICIAL=V"
if "%~1"=="w" set "HARNESS_INICIAL=W"
if "%~1"=="x" set "HARNESS_INICIAL=X"
if "%~1"=="y" set "HARNESS_INICIAL=Y"
if "%~1"=="z" set "HARNESS_INICIAL=Z"
goto :eof

:sin_ps1
echo [harness] No se encontro setup_harness.ps1 junto a este .cmd. 1>&2
echo           Buscado en: %HARNESS_PS1% 1>&2
echo           Corre este instalador desde la carpeta fuente del arnes. 1>&2
endlocal & exit /b 127

:sin_pwsh
echo [harness] No se encontro PowerShell en el PATH: ni pwsh.exe ni powershell.exe. 1>&2
echo           Remedio: instala PowerShell, o corre el instalador con Git Bash: 1>&2
echo               bash setup_harness.sh 1>&2
endlocal & exit /b 127
