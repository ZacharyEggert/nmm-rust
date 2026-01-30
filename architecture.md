# Nexus Mod Manager Architecture Documentation

## Overview

This document provides a comprehensive analysis of the Nexus Mod Manager (NMM) C# codebase architecture. The application is built on .NET Framework 4.6.1 with a WinForms UI, designed exclusively for Windows.

## Solution Structure

**Solution File**: `NexusClient.sln` (Visual Studio 2022)
**Total Projects**: ~75 C# projects
**Total C# Files**: ~1,481 files

### Core Projects Hierarchy

```
NexusClient.sln
├── Core Libraries
│   ├── NexusClient.Interface     # Core interfaces (IEnvironmentInfo, ISettings)
│   ├── ModManager.Interface      # Mod management contracts
│   ├── Transactions              # Transaction/rollback system
│   ├── Util                      # Shared utilities
│   └── UI                        # WinForms controls library
│
├── Main Application
│   ├── NexusClient               # Main WinForms application (WinExe)
│   └── NexusClientCLI            # Command-line interface
│
├── Scripting Engines
│   ├── Scripting                 # Base script execution engine
│   ├── Script Types/
│   │   ├── XmlScript             # XML-based mod configuration scripts
│   │   ├── ModScript             # Legacy Morrowind/Oblivion script format
│   │   └── CSharpScript          # C# scripting support
│
└── Game Modes (61 total)
    ├── GamebryoBase              # Bethesda game engine base
    ├── Skyrim, SkyrimSE, SkyrimVR, SkyrimGOG
    ├── Fallout3, FalloutNV, Fallout4, Fallout4VR
    ├── Oblivion, OblivionRemastered, Morrowind
    ├── Starfield
    └── [47 other game modes...]
```

## Core Interfaces

### 1. IGameModeDescriptor (ModManager.Interface/Games/IGameModeDescriptor.cs)

Provides static metadata about a game mode:

```csharp
public interface IGameModeDescriptor
{
    string Name { get; }                           // Display name (e.g., "Skyrim Special Edition")
    string ModeId { get; }                         // Unique identifier (e.g., "SkyrimSE")
    string[] GameExecutables { get; }              // Possible game executables
    string InstallationPath { get; }               // Primary mod installation path
    string SecondaryInstallationPath { get; }      // Secondary install path (if applicable)
    IEnumerable<string> PluginExtensions { get; }  // Plugin file extensions (.esp, .esm, .esl)
    IEnumerable<string> StopFolders { get; }       // Archive structure detection folders
    string ExecutablePath { get; }                 // Path to main game executable
    string[] OrderedCriticalPluginNames { get; }   // Critical plugins that cannot be reordered
    string[] OrderedOfficialPluginNames { get; }   // Official DLC plugins
    string[] OrderedOfficialUnmanagedPluginNames { get; }  // Unmanaged official plugins
    string PluginDirectory { get; }                // Where game plugins are stored
    Theme ModeTheme { get; }                       // UI theme for the game mode
}
```

### 2. IGameMode (ModManager.Interface/Games/IGameMode.cs)

Extends `IGameModeDescriptor` with runtime functionality:

```csharp
public interface IGameMode : IGameModeDescriptor, IDisposable
{
    // Environment
    IGameModeEnvironmentInfo GameModeEnvironmentInfo { get; }
    Version GameVersion { get; }
    IEnumerable<string> WritablePaths { get; }

    // Plugin Management
    bool UsesPlugins { get; }
    bool SupportsPluginAutoSorting { get; }
    ILoadOrderManager LoadOrderManager { get; }
    Int32 MaxAllowedActivePluginsCount { get; }
    bool PluginSorterInitialized { get; }

    // Mod Management
    bool RequiresModFileMerge { get; }
    string MergedFileName { get; }
    bool HasSecondaryInstallPath { get; }
    bool RequiresSpecialFileInstallation { get; }
    bool UsesModLoadOrder { get; }

    // Factory Methods
    IPluginFactory GetPluginFactory();
    IActivePluginLogSerializer GetActivePluginLogSerializer(IPluginOrderLog polPluginOrderLog);
    IPluginDiscoverer GetPluginDiscoverer();
    IPluginOrderLogSerializer GetPluginOrderLogSerializer();
    IPluginOrderValidator GetPluginOrderValidator();
    IGameSpecificValueInstaller GetGameSpecificValueInstaller(...);

    // Critical Plugin Management
    bool IsCriticalPlugin(Plugin plugin);
    string[] SortPlugins(IList<Plugin> plugins);

    // Path Adjustment (for legacy mod format compatibility)
    string GetModFormatAdjustedPath(IModFormat modFormat, string path, bool ignoreIfPresent);
}
```

### 3. IMod (ModManager.Interface/Mods/IMod.cs)

Represents a mod archive/package:

```csharp
public interface IMod : IModInfo, IScriptedMod
{
    // Identity
    string Filename { get; }
    string ModArchivePath { get; }
    IModFormat Format { get; }
    string ScreenshotPath { get; }

    // Read Transactions (for efficient batch file extraction)
    event CancelProgressEventHandler ReadOnlyInitProgressUpdated;
    void BeginReadOnlyTransaction(FileUtil fileUtil);
    void EndReadOnlyTransaction();

    // File Operations
    byte[] GetFile(string file);
    FileStream GetFileStream(string file);
    List<string> GetFileList();
    List<string> GetFileList(string folderPath, bool recurse);
    bool IsMatchingVersion();
}
```

### 4. IModInfo (ModManager.Interface/Mods/IModInfo.cs)

Mod metadata contract:

```csharp
public interface IModInfo
{
    string Id { get; set; }                    // Nexus Mods ID
    string DownloadId { get; set; }            // Download-specific ID
    DateTime? DownloadDate { get; set; }
    string ModName { get; }
    string FileName { get; }
    string HumanReadableVersion { get; set; }
    string LastKnownVersion { get; }
    bool? IsEndorsed { get; }
    Version MachineVersion { get; }
    string Author { get; }
    int CategoryId { get; }
    int CustomCategoryId { get; }
    string Description { get; }
    string InstallDate { get; set; }
    Uri Website { get; }
    ExtendedImage Screenshot { get; }
    bool UpdateWarningEnabled { get; }
    bool UpdateChecksEnabled { get; }
    int PlaceInModLoadOrder { get; set; }
    int NewPlaceInModLoadOrder { get; set; }

    void UpdateInfo(IModInfo modInfo, bool? overwriteAllValues);
}
```

### 5. IModFormat (ModManager.Interface/Mods/IModFormat.cs)

Archive format handler:

```csharp
public enum FormatConfidence
{
    Match = 3,       // Definitively this format
    Compatible = 2,  // Compatible with this format
    Convertible = 1, // Can be converted to this format
    Incompatible = 0 // Not this format
}

public interface IModFormat
{
    string Name { get; }
    string Id { get; }                     // "FOMod", "OMod", etc.
    string Extension { get; }              // File extension
    bool SupportsModCompression { get; }

    FormatConfidence CheckFormatCompliance(string path);
    IMod CreateMod(string path, IGameMode gameMode, bool isResetCachePath);
    IModCompressor GetModCompressor(IEnvironmentInfo environmentInfo);
}
```

### 6. IInstallLog (ModManager.Interface/ModManagement/InstallationLog/IInstallLog.cs)

Tracks all installed mod files and edits:

```csharp
public interface IInstallLog
{
    string OriginalValuesKey { get; }
    ReadOnlyObservableList<IMod> ActiveMods { get; }

    // Mod Tracking
    void AddActiveMod(IMod mod);
    void ReplaceActiveMod(IMod oldMod, IMod newMod);
    void RemoveMod(IMod modToRemove);
    string GetModKey(IMod mod);
    IEnumerable<KeyValuePair<IMod, IMod>> GetMismatchedVersionMods();

    // File Version Management (ownership tracking)
    void AddDataFile(IMod installingMod, string dataFilePath);
    void RemoveDataFile(IMod installingMod, string dataFilePath);
    IMod GetCurrentFileOwner(string path);
    IMod GetPreviousFileOwner(string path);
    IList<string> GetInstalledModFiles(IMod installer);
    IList<IMod> GetFileInstallers(string path);

    // INI Edit Management
    void AddIniEdit(IMod installingMod, string settingsFileName, string section, string key, string value);
    void RemoveIniEdit(IMod installingMod, string settingsFileName, string section, string key);
    IMod GetCurrentIniEditOwner(string settingsFileName, string section, string key);
    string GetPreviousIniValue(string settingsFileName, string section, string key);

    // Game Specific Values
    void AddGameSpecificValueEdit(IMod installingMod, string key, byte[] value);
    void RemoveGameSpecificValueEdit(IMod installingMod, string key);

    // Backup
    void Backup();
}
```

### 7. IVirtualModActivator (ModManager.Interface/ModManagement/VirtualModActivator/IVirtualModActivator.cs)

Virtual file system that creates symlinks/hardlinks for mod files:

```csharp
public interface IVirtualModActivator
{
    event EventHandler ModActivationChanged;

    // State
    bool MultiHDMode { get; }            // Multi-drive hardlink mode
    bool Initialized { get; }
    bool DisableLinkCreation { get; }
    string VirtualPath { get; }
    string HDLinkFolder { get; }
    ThreadSafeObservableList<IVirtualModLink> VirtualLinks { get; }
    ThreadSafeObservableList<IVirtualModInfo> VirtualMods { get; }
    IEnumerable<string> ActiveModList { get; }
    IGameMode GameMode { get; }

    // Lifecycle
    void Initialize();
    void Setup();
    void Reset();
    bool SaveList();
    List<IVirtualModLink> LoadList(string xmlFilePath);

    // File Link Operations
    string AddFileLink(IMod mod, string baseFilePath, bool isSwitching, bool isRestoring, int priority);
    void RemoveFileLink(string filePath, IMod mod);
    void UpdateLinkPriority(IVirtualModLink fileLink);
    bool PurgeLinks();

    // Mod Activation
    void EnableMod(IMod mod);
    void DisableMod(IMod mod);
    void FinalizeModActivation(IMod mod);
    void FinalizeModDeactivation(IMod mod);

    // INI Management
    void LogIniEdits(IMod mod, string settingsFileName, string section, string key, string value);
    void RestoreIniEdits();
    void PurgeIniEdits();
}
```

## Key Algorithms

### 1. Priority Resolution Algorithm

When multiple mods install the same file, NMM uses a priority-based ownership system:

```
For each file path:
    owners = InstallLog.GetFileInstallers(path)  // Stack of installers
    currentOwner = owners.Peek()                  // Top of stack = current owner

    When mod A installs file:
        if file exists:
            backup original or get previous owner
        owners.Push(modA)
        create symlink/hardlink to modA's version

    When mod A uninstalls:
        owners.Pop(modA)
        if owners.Any():
            previousOwner = owners.Peek()
            restore previousOwner's version
        else:
            restore original file (if backed up)
```

**Implementation**: `InstallLog.cs` uses `InstalledItemDictionary<string, object>` which maintains a stack of installers per file path.

### 2. Plugin Load Order (Gamebryo Games)

```
1. Critical plugins first (Skyrim.esm, Update.esm, etc.)
2. Official DLC plugins in defined order
3. User plugins by modification timestamp (default)
4. Or by explicit load order file (plugins.txt / loadorder.txt)

Validation rules:
- Masters must load before dependents
- Critical plugins cannot be reordered
- ESL (light) plugins have separate limit (FE index)
- Total plugin limit: 255 for ESP/ESM
```

**Implementation**: `GamebryoPluginOrderValidator.cs`, `PluginOrderManager.cs`

### 3. Transaction System

NMM uses a custom transaction system for rollback capability:

```csharp
using (TransactionScope scope = new TransactionScope())
{
    // File operations use TxFileManager
    txFileManager.Copy(source, dest);
    txFileManager.Delete(file);

    // Install log operations
    installLog.AddDataFile(mod, path);

    scope.Complete();  // Commit all changes
}
// If exception thrown, all operations roll back
```

**Implementation**: `Transactions` project, `InstallLog.TransactionEnlistment`

### 4. Archive Structure Detection

Mods can have various folder structures. NMM detects the "root" of mod content:

```
For Gamebryo games:
    StopFolders = ["Data", "Textures", "Meshes", "Scripts", ...]

    Algorithm:
    1. List all files in archive
    2. For each file path, check if any segment matches StopFolder
    3. If found, that's the mod root
    4. Adjust all paths relative to detected root
```

**Implementation**: `IGameModeDescriptor.StopFolders`, mod format handlers

## Data Formats

### 1. InstallLog.xml

Tracks all installed mods and their files:

```xml
<?xml version="1.0" encoding="utf-8"?>
<installLog fileVersion="0.5.0.0">
  <modList>
    <mod path="ModName.7z" key="mod_key_001">
      <version machineVersion="1.0.0.0">1.0</version>
      <name>Mod Display Name</name>
      <installDate>2024-01-15T10:30:00</installDate>
    </mod>
  </modList>

  <dataFiles>
    <file path="Data/Textures/texture.dds">
      <installingMods>
        <mod key="mod_key_001"/>
        <mod key="mod_key_002"/>  <!-- Stack: 002 is current owner -->
      </installingMods>
    </file>
  </dataFiles>

  <iniEdits>
    <ini file="Skyrim.ini" section="Display" key="fShadowDistance">
      <installingMods>
        <mod key="mod_key_001">4000.0</mod>
      </installingMods>
    </ini>
  </iniEdits>

  <gameSpecificEdits>
    <!-- Base64-encoded game-specific data -->
  </gameSpecificEdits>
</installLog>
```

### 2. VirtualModConfig.xml

Virtual file system state:

```xml
<?xml version="1.0" encoding="utf-8"?>
<virtualModActivator fileVersion="0.3.0.0">
  <modList>
    <modInfo modId="12345"
             downloadId="67890"
             modName="Example Mod"
             modFileName="ExampleMod.7z"
             modFilePath="C:\NMM\Skyrim\Mods"
             FileVersion="1.0">
      <fileLink realPath="ExampleMod/Data/textures/example.dds"
                virtualPath="Data/textures/example.dds">
        <linkPriority>0</linkPriority>
        <isActive>true</isActive>
      </fileLink>
    </modInfo>
  </modList>
</virtualModActivator>
```

### 3. plugins.txt (Bethesda Games)

Active plugin list:

```
# This file is used by the game to determine which plugins to load
*Skyrim.esm
*Update.esm
*Dawnguard.esm
*HearthFires.esm
*Dragonborn.esm
MyMod.esp
AnotherMod.esp
```

(`*` prefix indicates the plugin is active)

### 4. loadorder.txt (Bethesda Games)

Full load order including inactive:

```
Skyrim.esm
Update.esm
Dawnguard.esm
HearthFires.esm
Dragonborn.esm
MyMod.esp
InactiveMod.esp
AnotherMod.esp
```

## Scripting System

### XmlScript (Primary Script Type)

Versions: 1.0, 2.0, 3.0, 4.0, 5.0

Structure:
```xml
<config xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
        xsi:noNamespaceSchemaLocation="http://qconsulting.ca/fo3/ModConfig5.0.xsd">

  <moduleName>Mod Configuration</moduleName>
  <moduleImage path="fomod/screenshot.png"/>

  <requiredInstallFiles>
    <file source="core/file.esp"/>
  </requiredInstallFiles>

  <installSteps order="Explicit">
    <installStep name="Choose Options">
      <optionalFileGroups order="Explicit">
        <group name="Textures" type="SelectExactlyOne">
          <plugins order="Explicit">
            <plugin name="2K Textures">
              <description>High resolution textures</description>
              <image path="fomod/2k_preview.png"/>
              <files>
                <folder source="optional/2k" destination="Data/Textures"/>
              </files>
              <typeDescriptor>
                <type name="Recommended"/>
              </typeDescriptor>
            </plugin>
          </plugins>
        </group>
      </optionalFileGroups>
    </installStep>
  </installSteps>

  <conditionalFileInstalls>
    <patterns>
      <pattern>
        <dependencies operator="And">
          <fileDependency file="SKSE/skse_loader.exe" state="Active"/>
        </dependencies>
        <files>
          <file source="optional/skse_plugin.dll" destination="Data/SKSE/Plugins"/>
        </files>
      </pattern>
    </patterns>
  </conditionalFileInstalls>
</config>
```

### Script Execution Flow

```
1. Detect script type (XmlScript, ModScript, CSharpScript)
2. Parse script into in-memory representation
3. Evaluate conditions (file dependencies, game version, etc.)
4. Present UI for user choices
5. Execute file installations based on selections
6. Log all changes to InstallLog
```

**Implementation**:
- `XmlScriptType.cs` - Type registration and factory
- `XmlScriptExecutor.cs` - Execution engine
- `ConditionStateManager.cs` - Condition evaluation
- `Parser10.cs` through `Parser50.cs` - Version-specific parsers

## Game Mode Architecture

### Base Classes

```
GameModeBase (abstract)
    │
    ├── GamebryoGameModeBase (abstract)
    │       │
    │       ├── SkyrimGameMode
    │       ├── SkyrimSEGameMode
    │       ├── Fallout4GameMode
    │       ├── OblivionGameMode
    │       └── [other Bethesda games...]
    │
    └── [Non-Gamebryo bases]
            │
            ├── Witcher3GameMode
            ├── Cyberpunk2077GameMode
            └── BaldursGate3GameMode
```

### GamebryoGameModeBase Features

Provides common functionality for Bethesda games:

```csharp
public abstract class GamebryoGameModeBase : GameModeBase
{
    // Plugin Management
    private GamebryoPluginFactory m_pgfPluginFactory;
    private PluginSorter PluginSorter;
    protected PluginOrderManager PluginOrderManager;

    // Settings
    public GamebryoSettingsFiles SettingsFiles { get; }  // INI files

    // Constants
    private const int m_intMaxAllowedPlugins = 255;

    // Abstract Members
    protected abstract string[] ScriptExtenderExecutables { get; }
    public abstract string UserGameDataPath { get; }
    protected abstract GamebryoSettingsFiles CreateSettingsFileContainer();

    // Plugin file types requiring hardlinks
    public override bool HardlinkRequiredFilesType(string fileName)
    {
        // .esp, .esl, .esm, .bsa, .mp3, .BGSM, .BGEM, .wav, .ogg, .xwm
    }
}
```

### Game Mode Registry

Games are discovered via assembly scanning:

```csharp
// Each game mode project exports a descriptor class
[Export(typeof(IGameModeDescriptor))]
public class SkyrimSEGameModeDescriptor : GameModeDescriptorBase
{
    public override string ModeId => "SkyrimSE";
    public override string Name => "Skyrim Special Edition";
    // ...
}
```

## Complete Game Mode List

### Gamebryo/Creation Engine (Bethesda)
| Game Mode | ID | Plugin Extensions |
|-----------|----|--------------------|
| The Elder Scrolls III: Morrowind | Morrowind | .esp, .esm |
| The Elder Scrolls IV: Oblivion | Oblivion | .esp, .esm |
| Oblivion Remastered | OblivionRemastered | .esp, .esm, .esl |
| The Elder Scrolls V: Skyrim | Skyrim | .esp, .esm |
| Skyrim Special Edition | SkyrimSE | .esp, .esm, .esl |
| Skyrim VR | SkyrimVR | .esp, .esm, .esl |
| Skyrim GOG | SkyrimGOG | .esp, .esm, .esl |
| Enderal | Enderal | .esp, .esm |
| Enderal Special Edition | EnderalSE | .esp, .esm, .esl |
| Fallout 3 | Fallout3 | .esp, .esm |
| Fallout: New Vegas | FalloutNV | .esp, .esm |
| Fallout 4 | Fallout4 | .esp, .esm, .esl |
| Fallout 4 VR | Fallout4VR | .esp, .esm |
| Starfield | Starfield | .esp, .esm, .esl |

### Other Games
| Game | ID | Uses Plugins |
|------|----|--------------|
| Baldur's Gate 3 | BaldursGate3 | No |
| Cyberpunk 2077 | Cyberpunk2077 | No |
| The Witcher 3 | Witcher3 | No |
| The Witcher 2 | Witcher2 | No |
| Dragon Age: Origins | DragonAge | No |
| Dragon Age 2 | DragonAge2 | No |
| Dragon's Dogma | DragonsDogma | No |
| Dark Souls | DarkSouls | No |
| Dark Souls 2 | DarkSouls2 | No |
| Hogwarts Legacy | HogwartsLegacy | No |
| Monster Hunter: World | MonsterHunterWorld | No |
| Mount & Blade II: Bannerlord | MountAndBlade2Bannerlord | No |
| No Man's Sky | NoMansSky | No |
| Stardew Valley | StardewValley | No |
| Starbound | Starbound | No |
| The Sims 4 | Sims4 | No |
| State of Decay | StateOfDecay | No |
| Subnautica | Subnautica | No |
| Subnautica: Below Zero | SubnauticaBelowZero | No |
| The Elder Scrolls Online | TESO | No |
| War Thunder | WarThunder | No |
| World of Tanks | WorldOfTanks | No |
| XCOM 2 | XCOM2 | No |
| X Rebirth | XRebirth | No |
| Grimrock | Grimrock | No |
| Breaking Wheel | BreakingWheel | No |

## Data Flow Diagrams

### Mod Installation Flow

```
User drops mod archive
        │
        ▼
┌─────────────────────┐
│  Format Detection   │ ◄── IModFormat.CheckFormatCompliance()
│  (FOMod, OMod, 7z)  │
└─────────────────────┘
        │
        ▼
┌─────────────────────┐
│   Script Detection  │ ◄── Check for fomod/ModuleConfig.xml, script.xml
│   & Parsing         │
└─────────────────────┘
        │
        ▼
┌─────────────────────┐
│    User Selection   │ ◄── XmlScript UI shows options
│    (if scripted)    │
└─────────────────────┘
        │
        ▼
┌─────────────────────┐
│  Transaction Begin  │
└─────────────────────┘
        │
        ├──────────────────────────────────────┐
        │                                      │
        ▼                                      ▼
┌─────────────────────┐              ┌─────────────────────┐
│   Extract Files     │              │   Update InstallLog │
│   to VirtualInstall │              │   (file ownership)  │
└─────────────────────┘              └─────────────────────┘
        │                                      │
        └──────────────────────────────────────┤
                                               │
        ▼                                      │
┌─────────────────────┐                        │
│  Create Symlinks/   │ ◄──────────────────────┘
│  Hardlinks in Game  │
└─────────────────────┘
        │
        ▼
┌─────────────────────┐
│  Update Plugin List │ ◄── If game uses plugins
│  (activate .esp)    │
└─────────────────────┘
        │
        ▼
┌─────────────────────┐
│  Transaction Commit │
└─────────────────────┘
```

### Profile Switch Flow

```
User selects profile
        │
        ▼
┌─────────────────────┐
│  Load Profile       │ ◄── VirtualModConfig.xml, plugins.txt
│  Configuration      │
└─────────────────────┘
        │
        ▼
┌─────────────────────┐
│  Disable Current    │
│  Active Mods        │ ◄── Remove all symlinks
└─────────────────────┘
        │
        ▼
┌─────────────────────┐
│  Enable Profile     │
│  Mods in Order      │ ◄── Create symlinks per priority
└─────────────────────┘
        │
        ▼
┌─────────────────────┐
│  Restore INI Edits  │ ◄── Apply profile's INI settings
└─────────────────────┘
        │
        ▼
┌─────────────────────┐
│  Update Plugin      │ ◄── Restore plugins.txt
│  Load Order         │
└─────────────────────┘
```

## Thread Safety

Key thread-safe collections used:

```csharp
ThreadSafeObservableList<T>  // Observable list with locking
ConcurrentDictionary<K,V>   // Thread-safe dictionary
```

Critical sections protected by:
- `lock` statements on shared resources
- `Mutex` for cross-process coordination (single instance)
- Transaction isolation for file operations

## Windows-Specific Dependencies

| Feature | Windows API/Dependency |
|---------|------------------------|
| Symlinks | `CreateSymbolicLink` (kernel32.dll) |
| Hardlinks | `CreateHardLink` (kernel32.dll) |
| Registry | Game path detection via registry |
| WinForms | UI framework |
| .NET Framework 4.6.1 | Runtime |

## Critical Implementation Notes

1. **Symlink Permissions**: Windows requires Developer Mode or admin privileges for symlinks. NMM falls back to hardlinks when symlinks fail.

2. **Multi-HD Mode**: When game and mods are on different drives, hardlinks cannot work. NMM uses a "link folder" on the same drive as the game.

3. **File Locking**: Game files may be locked while game is running. NMM must handle this gracefully.

4. **Unicode Paths**: All path operations must handle Unicode characters properly.

5. **Path Length**: Windows has a 260-character path limit by default. Long mod paths can cause issues.

6. **Case Sensitivity**: Windows is case-insensitive but preserves case. File tracking must be case-insensitive.
