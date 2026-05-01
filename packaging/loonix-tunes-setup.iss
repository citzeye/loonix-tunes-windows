[Setup]
AppName=Loonix Tunes
AppVersion=2.0.0
DefaultDirName={autopf}\Loonix Tunes
DefaultGroupName=Loonix Tunes
OutputDir=..\Output
OutputBaseFilename=LoonixTunesWin64v2
Compression=lzma2/ultra64
InternalCompressLevel=ultra
SolidCompression=yes
LZMADictionarySize=24
LZMANumBlockThreads=4
WizardStyle=modern
PrivilegesRequired=admin
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
UninstallDisplayIcon={app}\LoonixTunesWin64v2.exe

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

#define DistFolder ".\dist\vendor"

[Files]
; 1. Main app: Copy app-core.bin but rename to .exe on install
Source: "{#DistFolder}\app-core.bin"; DestDir: "{app}"; DestName: "LoonixTunesWin64v2.exe"; Flags: ignoreversion

; 2. All other files (DLLs, QML, Qt), EXCLUDE app-core.bin (already handled above)
Source: "{#DistFolder}\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs; Excludes: "app-core.bin"

[Icons]
Name: "{group}\Loonix Tunes"; Filename: "{app}\LoonixTunesWin64v2.exe"
Name: "{autodesktop}\Loonix Tunes"; Filename: "{app}\LoonixTunesWin64v2.exe"; Tasks: desktopicon

[Tasks]
Name: "desktopicon"; Description: "Create a &desktop shortcut"; GroupDescription: "Additional icons:"

[Run]
Filename: "{app}\LoonixTunesWin64v2.exe"; Description: "Launch Loonix Tunes"; Flags: nowait skipifsilent
