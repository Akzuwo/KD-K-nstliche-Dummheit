#ifndef StageDir
  #error StageDir required
#endif
#ifndef OutputDir
  #error OutputDir required
#endif
#ifndef AppVersion
  #error AppVersion required
#endif
[Setup]
AppId={{49F17DE8-58CF-4BDF-91B5-EBB6F81E4638}
AppName=KD Desktop
AppVersion={#AppVersion}
AppVerName=KD Desktop v{#AppVersion}
AppPublisher=KD Projekt
DefaultDirName={localappdata}\Programs\KD Desktop
DefaultGroupName=KD Desktop
UsePreviousGroup=no
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
OutputDir={#OutputDir}
OutputBaseFilename=KD-Desktop-{#AppVersion}-Setup
SetupIconFile=..\assets\kd.ico
UninstallDisplayIcon={app}\KD.exe
Compression=lzma2/fast
SolidCompression=yes
WizardStyle=modern
CloseApplications=yes
CloseApplicationsFilter=KD.exe
RestartApplications=no
DisableWelcomePage=no
LicenseFile=..\LICENSE.txt
InfoAfterFile=..\INSTALLATION.txt
MinVersion=10.0.17763

[Languages]
Name: "german"; MessagesFile: "compiler:Languages\German.isl"

[Tasks]
Name: "desktopicon"; Description: "Desktop-Verknüpfung erstellen"; Flags: unchecked

[Files]
Source: "{#StageDir}\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs; Excludes: "MicrosoftEdgeWebView2RuntimeInstallerX64.exe"
Source: "{#StageDir}\MicrosoftEdgeWebView2RuntimeInstallerX64.exe"; DestDir: "{tmp}"; Flags: deleteafterinstall; Check: NeedsWebView

[Icons]
Name: "{group}\KD Desktop"; Filename: "{app}\KD.exe"; WorkingDir: "{app}"
Name: "{autodesktop}\KD Desktop"; Filename: "{app}\KD.exe"; Tasks: desktopicon

[InstallDelete]
; Remove only this application's previous shortcut names during the rename.
Type: files; Name: "{userprograms}\Künstliche Dummheit\Künstliche Dummheit.lnk"
Type: files; Name: "{userdesktop}\Künstliche Dummheit.lnk"

[INI]
Filename: "{app}\kd-install.ini"; Section: "KD Desktop"; Key: "Version"; String: "{#AppVersion}"; Flags: uninsdeletesection

[UninstallDelete]
Type: files; Name: "{app}\kd-install.ini"

[Run]
Filename: "{tmp}\MicrosoftEdgeWebView2RuntimeInstallerX64.exe"; Parameters: "/silent /install"; StatusMsg: "Lokale WebView2-Laufzeit wird installiert …"; Flags: waituntilterminated; Check: NeedsWebView
Filename: "{app}\KD.exe"; Description: "KD Desktop starten"; Flags: nowait postinstall skipifsilent

[Code]
function OpenProcess(DesiredAccess: Cardinal; InheritHandle: Boolean; ProcessId: Cardinal): THandle;
  external 'OpenProcess@kernel32.dll stdcall';
function WaitForSingleObject(Handle: THandle; Milliseconds: Cardinal): Cardinal;
  external 'WaitForSingleObject@kernel32.dll stdcall';
function CloseHandle(Handle: THandle): Boolean;
  external 'CloseHandle@kernel32.dll stdcall';

function InitializeSetup: Boolean;
var ParentPid: Integer; ParentHandle: THandle;
begin
  ParentPid := StrToIntDef(ExpandConstant('{param:KDWaitPid|0}'), 0);
  Result := True;
  if ParentPid > 0 then begin
    ParentHandle := OpenProcess($00100000, False, ParentPid);
    if ParentHandle <> 0 then begin
      Result := WaitForSingleObject(ParentHandle, 30000) = 0;
      CloseHandle(ParentHandle);
    end;
  end;
end;

function NeedsWebView: Boolean;
var Version: String;
begin
  Result := not (RegQueryStringValue(HKCU, 'Software\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}', 'pv', Version) or
    RegQueryStringValue(HKLM32, 'Software\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}', 'pv', Version));
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
var Purge: Boolean; DataDir: String;
begin
  if CurUninstallStep = usPostUninstall then begin
    Purge := False;
    if not UninstallSilent then
      Purge := MsgBox('Auch alle lokalen KD-Chats, Einstellungen und importierten Modelle dauerhaft löschen?' + #13#10 + #13#10 +
        'Wähle Nein, um deine Daten für eine spätere Installation zu behalten.' + #13#10 +
        'Original-ZIPs und andere KD-Projektordner bleiben erhalten.', mbConfirmation, MB_YESNO or MB_DEFBUTTON2) = IDYES;
    if Purge then begin
      DataDir := ExpandConstant('{localappdata}\KD Desktop');
      DelTree(DataDir, True, True, True);
    end;
  end;
end;
