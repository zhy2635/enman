[Setup]
AppName=Enman
AppVersion=1.0
DefaultDirName={autopf}\Enman
DefaultGroupName=Enman
OutputDir=.
OutputBaseFilename=enman-setup
Compression=lzma
SolidCompression=yes
PrivilegesRequired=admin
ArchitecturesAllowed=x64
ArchitecturesInstallIn64BitMode=x64
WizardStyle=modern

; 定义英语语言
[Languages]
Name: "en"; MessagesFile: "compiler:Default.isl"

[CustomMessages]
; English
en.CliLanguageSelection=Select Command Line Interface Language
en.CliSelectLanguage=Please select the default language for command line prompts:
en.EnglishOption=English
en.ChineseOption=中文 (Chinese)

[Tasks]
Name: "addtopath"; Description: "Add Enman to system PATH (recommended)"; GroupDescription: "System Settings:";

[Files]
Source: "enman.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "em.exe"; DestDir: "{app}"; Flags: ignoreversion

[Registry]
Root: HKLM; Subkey: "SYSTEM\CurrentControlSet\Control\Session Manager\Environment"; \
    ValueType: expandsz; ValueName: "PATH"; ValueData: "{olddata};{app}"; \
    Check: ShouldAddToPath

[Code]
var
  LangPage: TInputOptionWizardPage;

function ShouldAddToPath: Boolean;
begin
  Result := WizardIsTaskSelected('addtopath');
end;

procedure InitializeWizard;
begin
  ; 创建语言选择页面
  LangPage := CreateInputOptionPage(wpWelcome,
    CustomMessage('CliLanguageSelection'),
    CustomMessage('CliSelectLanguage'),
    '', True, False);
  LangPage.Add(CustomMessage('EnglishOption'));
  LangPage.Add(CustomMessage('ChineseOption'));
  
  ; 设置默认选择（根据系统语言）
  if GetUILanguage() = 2052 then
    LangPage.SelectedValueIndex := 1  // 中文
  else
    LangPage.SelectedValueIndex := 0; // 英文
end;

function NextButtonClick(CurPageID: Integer): Boolean;
begin
  Result := True;
  
  if CurPageID = wpWelcome then
  begin
    // 根据用户选择的语言设置安装程序语言
    if LangPage.SelectedValueIndex = 1 then
      SetPreviousData('Language', 'chinese')
    else
      SetPreviousData('Language', 'english');
  end;
end;

// 创建配置文件以设置默认语言
procedure CreateConfigFile(const AppDir: String; const LanguageCode: String);
var
  ConfigPath: String;
  ConfigContent: String;
begin
  ConfigPath := ExpandConstant('{userappdata}\Enman\config.toml');
  
  // 确保配置目录存在
  if not ForceDirectories(ExtractFilePath(ConfigPath)) then
  begin
    MsgBox('Could not create configuration directory.', mbError, MB_OK);
    Exit;
  end;

  // 根据选择的语言设置配置内容
  if LanguageCode = 'chinese' then
    ConfigContent := 'language = "zh-CN"' + #13#10
  else
    ConfigContent := 'language = "en-US"' + #13#10;
    
  // 写入配置文件
  if not SaveStringToFile(ConfigPath, ConfigContent, False) then
    MsgBox('Could not create configuration file.', mbError, MB_OK);
end;

// 可选：安装完成后弹出提示（非必须）
procedure CurStepChanged(CurStep: TSetupStep);
var
  msgText: String;
  selectedLang: String;
begin
  if CurStep = ssPostInstall then
  begin
    // 使用在欢迎页面选择的语言设置
    if LangPage.SelectedValueIndex = 1 then
      selectedLang := 'chinese'
    else
      selectedLang := 'english';
    
    // 创建配置文件
    CreateConfigFile(WizardDirValue, selectedLang);
    
    if WizardIsTaskSelected('addtopath') then
    begin
      if selectedLang = 'chinese' then
        msgText := 'Enman 已安装并添加到 PATH。' + #13#13 + 
                   '请打开一个新的终端来使用 "enman" 命令。' + #13#13 +
                   '命令行界面将默认使用中文显示。'
      else
        msgText := 'Enman has been installed and added to PATH.' + #13#13 + 
                   'Open a NEW terminal to use the "enman" command.' + #13#13 +
                   'Command line interface will display in English by default.';
      
      MsgBox(msgText, mbInformation, MB_OK);
    end
    else
    begin
      if selectedLang = 'chinese' then
        msgText := 'Enman 已安装到：' + #13 + ExpandConstant('{app}') + #13#13 + 
                   '要从任何位置使用它，请手动将此文件夹添加到您的 PATH。' + #13#13 +
                   '命令行界面将默认使用中文显示。'
      else
        msgText := 'Enman has been installed to:' + #13 + ExpandConstant('{app}') + #13#13 + 
                   'To use it from anywhere, manually add this folder to your PATH.' + #13#13 +
                   'Command line interface will display in English by default.';
      
      MsgBox(msgText, mbInformation, MB_OK);
    end;
  end;
end;
