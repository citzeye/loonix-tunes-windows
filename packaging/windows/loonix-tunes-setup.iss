[Setup]
AppName=Loonix Tunes
AppVersion=2.0.0
DefaultDirName={autopf}\Loonix Tunes
DefaultGroupName=Loonix Tunes
OutputDir=..\..\Output
OutputBaseFilename=loonix-tunes-2.0.0-x64
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
PrivilegesRequired=admin
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
UninstallDisplayIcon={app}\loonix-tunes.exe

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

#define DistFolder ".\dist"

[Files]
Source: "{#DistFolder}\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs

[Icons]
Name: "{group}\Loonix Tunes"; Filename: "{app}\loonix-tunes.exe"
Name: "{autodesktop}\Loonix Tunes"; Filename: "{app}\loonix-tunes.exe"; Tasks: desktopicon

[Tasks]
Name: "desktopicon"; Description: "Create a &desktop shortcut"; GroupDescription: "Additional icons:"

[Run]
Filename: "{app}\loonix-tunes.exe"; Description: "Launch Loonix Tunes"; Flags: nowait skipifsilent
