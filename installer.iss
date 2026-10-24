[Setup]
AppId={{A1B2C3D4-1111-2222-3333-444455556666}
AppName=Lechapes
AppVersion=0.5.0
AppVerName=Lechapes 0.5.0
AppPublisher=Johanes-mg
AppPublisherURL=https://johanes-mg.github.io
AppSupportURL=https://johanes-mg.github.io
DefaultDirName={autopf}\Lechapes
DefaultGroupName=Lechapes
DisableProgramGroupPage=yes
OutputDir=Installeur
OutputBaseFilename=LechapesSetup
SetupIconFile=images\logo.ico
Compression=lzma2/ultra64
SolidCompression=yes
LZMAUseSeparateProcess=yes
WizardStyle=modern
ArchitecturesInstallIn64BitMode=x64compatible
ArchitecturesAllowed=x64compatible
PrivilegesRequired=admin
UninstallDisplayIcon={app}\lechapes.exe
DisableDirPage=no
DisableReadyPage=no
UsePreviousAppDir=yes
VersionInfoVersion=0.5.0
VersionInfoCompany=Johanes-mg
VersionInfoDescription=Lechapes 0.5.0 - Installation
VersionInfoProductName=Lechapes
VersionInfoProductVersion=0.5.0

[Languages]
Name: "french"; MessagesFile: "compiler:Languages\French.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "target\release\lechapes.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "images\*.png"; DestDir: "{app}\images"; Flags: ignoreversion
Source: "images\*.ico"; DestDir: "{app}\images"; Flags: ignoreversion

[Icons]
Name: "{group}\Lechapes"; Filename: "{app}\lechapes.exe"; IconFilename: "{app}\lechapes.exe"; IconIndex: 0; WorkingDir: "{app}"
Name: "{group}\{cm:UninstallProgram,Lechapes}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\Lechapes"; Filename: "{app}\lechapes.exe"; Tasks: desktopicon; IconFilename: "{app}\lechapes.exe"; IconIndex: 0; WorkingDir: "{app}"

[Run]
Filename: "{app}\lechapes.exe"; Description: "{cm:LaunchProgram,Lechapes}"; Flags: nowait postinstall skipifsilent
