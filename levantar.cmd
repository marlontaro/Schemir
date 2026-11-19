@echo off
setlocal
title Schemir - desarrollo

rem Se levanta desde la carpeta de este archivo, asi que da igual desde donde
rem se haga doble clic.
cd /d "%~dp0src-tauri"

rem Cargo se instala en %USERPROFILE%\.cargo\bin, pero una terminal (o la propia
rem app que la aloja) abierta ANTES de instalar Rust arrastra el PATH viejo y no
rem lo ve. Si falta, se agrega solo para esta ventana.
where cargo >nul 2>nul || set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"

where cargo >nul 2>nul
if errorlevel 1 (
  echo.
  echo No encuentro cargo.exe.
  echo Instala Rust desde https://rustup.rs y vuelve a ejecutar este archivo.
  echo.
  pause
  exit /b 1
)

echo Compilando y levantando Schemir...
echo La ventana de la aplicacion se abre sola. Cierrala para terminar.
echo.

rem Lo que se le pase a este .cmd llega a cargo: por ejemplo  levantar.cmd --release
cargo run %*

if errorlevel 1 (
  echo.
  echo === Schemir termino con error. El detalle esta arriba. ===
  pause
  exit /b 1
)
