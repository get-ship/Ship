@echo off
REM Ship launcher — runs from the Start Menu "Ship" shortcut.
REM
REM Ensures the Task Scheduler autostart task has kicked the daemon,
REM then opens the browser at the dashboard. The `start "" ...` form
REM dispatches the URL to the user's default browser without leaving
REM a console window behind.

schtasks /Run /TN ShipDaemon >nul 2>&1
start "" "http://127.0.0.1:9315/"
