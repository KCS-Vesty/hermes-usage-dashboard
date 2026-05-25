@echo off
REM Hermes Usage Dashboard - Quick Start
REM Starts InfluxDB and the Dashboard app

echo Starting InfluxDB v2.7.12...
start "InfluxDB" /B "%USERPROFILE%\influxdb2\influxd.exe" --reporting-disabled

echo Waiting for InfluxDB to be ready...
:wait
timeout /t 2 /nobreak >nul
curl -s -o nul "http://127.0.0.1:8086/health" 2>nul
if errorlevel 1 goto wait

echo InfluxDB is running.
echo.
echo Starting Hermes Usage Dashboard...
start "" "%USERPROFILE%\hermes-usage-dashboard\target\release\hermes-dashboard-tauri.exe"
echo.
echo Dashboard launched!
echo.
echo InfluxDB is available at:  http://127.0.0.1:8086
echo Dashboard is at:          Hermes Usage Dashboard window
echo.
echo Configure the dashboard with:
echo   URL:    http://127.0.0.1:8086
echo   Org:    hermes
echo   Bucket: hermes_usage
echo   Token:  (get from InfluxDB admin UI at http://127.0.0.1:8086)
echo.
echo InfluxDB Admin UI login:
echo   Username: hermes
echo   Password: password123
