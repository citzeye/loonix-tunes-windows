[Setup]
AppName=Loonix Tunes
AppVersion=2.0.0
DefaultDirName={autopf}\Loonix Tunes
DefaultGroupName=Loonix Tunes
OutputDir=..\Output
OutputBaseFilename=LoonixTunesWin64v2
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
PrivilegesRequired=admin
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
UninstallDisplayIcon={app}\LoonixTunesWin64v2.exe

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

#define DistFolder ".\dist"

[Files]
Source: "{#DistFolder}\LoonixTunesWin64v2.exe"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs

[Icons]
Name: "{group}\Loonix Tunes"; Filename: "{app}\LoonixTunesWin64v2.exe"
Name: "{autodesktop}\Loonix Tunes"; Filename: "{app}\LoonixTunesWin64v2.exe"; Tasks: desktopicon

[Tasks]
Name: "desktopicon"; Description: "Create a &desktop shortcut"; GroupDescription: "Additional icons:"

[Run]
Filename: "{app}\LoonixTunesWin64v2.exe"; Description: "Launch Loonix Tunes"; Flags: nowait skipifsilent
