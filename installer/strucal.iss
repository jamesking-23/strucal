#ifndef AppVersion
  #define AppVersion "0.1.0"
#endif
[Setup]
AppId={{B7C4E2A1-5D3F-4E8A-9C61-2F0A7D5E9B14}
AppName=StruCal
AppVersion={#AppVersion}
AppVerName=StruCal {#AppVersion}
AppPublisher=StruCal
DefaultDirName={autopf}\StruCal
DefaultGroupName=StruCal
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
OutputDir=..\dist
OutputBaseFilename=StruCal-Setup-x64
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
UninstallDisplayIcon={app}\StruCal.exe
CloseApplications=yes
DisableProgramGroupPage=yes

[Tasks]
Name: "desktopicon"; Description: "Create a &desktop shortcut"; GroupDescription: "Shortcuts:"

[Files]
Source: "..\target\release\strucal-desktop.exe"; DestDir: "{app}"; DestName: "StruCal.exe"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\StruCal"; Filename: "{app}\StruCal.exe"
Name: "{autodesktop}\StruCal"; Filename: "{app}\StruCal.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\StruCal.exe"; Description: "Launch StruCal"; Flags: nowait postinstall skipifsilent
Filename: "{app}\StruCal.exe"; Flags: nowait; Check: WizardSilent
