[Setup]
AppName=File Monitor Agent
AppVersion=0.1.1
AppPublisher=Rafal Curzydlo
DefaultDirName={autopf}\File Monitor Agent
DefaultGroupName=File Monitor Agent
; Używamy ścieżki względnej dla folderu wyjściowego
OutputDir=Instalator
OutputBaseFilename=FileMonitorAgent_Setup
Compression=lzma
SolidCompression=yes
PrivilegesRequired=admin

[Files]
; Używamy ścieżki względnej do skompilowanego pliku
Source: "target\release\file-monitor-agent.exe"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\File Monitor Agent"; Filename: "{app}\file-monitor-agent.exe"
Name: "{autodesktop}\File Monitor Agent"; Filename: "{app}\file-monitor-agent.exe"; Tasks: desktopicon

[Tasks]
Name: "desktopicon"; Description: "Utworz skrot na pulpicie"; GroupDescription: "Dodatkowe ikony:"; Flags: unchecked