@echo off
rem ===========================================================================
rem  Punto de entrada del Harness desde cmd.exe. Homologado con `harness_cli`
rem  (sh) y `harness_cli.ps1`: despacha SIEMPRE al binario nativo harness.exe.
rem
rem  No pasa por PowerShell a proposito: es el comando del dia a dia y no tiene
rem  por que pagar el arranque de otro shell ni depender de la ExecutionPolicy.
rem
rem  Feature #25: cubre la mitad que `doctor` NO puede cubrir por definicion:
rem  un doctor que vive en el binario no puede diagnosticar un binario ausente
rem  ni uno tan viejo que no conozca el subcomando. Los dos casos se traducen
rem  al mismo remedio en vez de dejar salir el error de clap.
rem
rem  La salida NO se captura: con el hub sin responder `close` tarda ~90s, y un
rem  wrapper que buferea deja al usuario mirando una pantalla muda.
rem ===========================================================================
setlocal

set "HARNESS_DIR=%~dp0"
set "HARNESS_BIN=%HARNESS_DIR%harness.exe"

if not exist "%HARNESS_BIN%" goto sin_binario

"%HARNESS_BIN%" %*
set "HARNESS_RC=%ERRORLEVEL%"

rem 2 es el codigo de uso invalido de clap. Solo entonces vale la pena
rem preguntar, y la pregunta es barata: `help <sub>` no ejecuta el subcomando.
if not "%HARNESS_RC%"=="2" goto fin
if "%~1"=="" goto fin
"%HARNESS_BIN%" help %1 >nul 2>&1
if "%ERRORLEVEL%"=="0" goto fin
echo. 1>&2
echo [harness_cli] El binario instalado no conoce el subcomando %1: es mas viejo 1>&2
echo               que los scripts que lo invocan, tipico de un git pull sin 1>&2
echo               re-correr el instalador. 1>&2
echo               Remedio: setup_harness.cmd 1>&2

:fin
endlocal & exit /b %HARNESS_RC%

:sin_binario
echo [harness_cli] Binario harness.exe no encontrado en %HARNESS_DIR% 1>&2
echo               Remedio: setup_harness.cmd 1>&2
echo               requiere rust/cargo disponible para compilar el binario 1>&2
endlocal & exit /b 127
