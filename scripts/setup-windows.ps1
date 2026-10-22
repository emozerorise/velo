param([string]$Archive, [switch]$VerifyOnly)
$ErrorActionPreference = 'Stop'
$root = Split-Path $PSScriptRoot -Parent
$pin = Get-Content "$PSScriptRoot/windows-mpv.json" -Raw | ConvertFrom-Json
$dest = Join-Path $root 'src-tauri/windows-deps'
New-Item -ItemType Directory -Force $dest | Out-Null
if (!$Archive) {
    $Archive = Join-Path $dest 'mpv.7z'
    if (!(Test-Path $Archive)) { Invoke-WebRequest $pin.url -OutFile $Archive }
}
if ((Get-FileHash $Archive -Algorithm SHA256).Hash.ToLowerInvariant() -ne $pin.sha256) {
    throw 'libmpv checksum mismatch; refusing to extract or execute downloaded content'
}
if ($VerifyOnly) { return }
& 7z x -y "-o$dest" $Archive
if ($LASTEXITCODE -ne 0) { throw 'libmpv extraction failed' }
$dll = Join-Path $dest 'libmpv-2.dll'
if (!(Test-Path $dll)) { throw 'libmpv-2.dll missing' }
# Upstream ships a MinGW import library, not an MSVC library or .def file.
$exports = & dumpbin /nologo /exports $dll
if ($LASTEXITCODE -ne 0) { throw 'Cannot inspect libmpv exports' }
$names = @($exports | ForEach-Object {
    if ($_ -match '^\s+\d+\s+[0-9A-F]+\s+[0-9A-F]+\s+(mpv_\w+)\s*$') { $Matches[1] }
})
if ($names -notcontains 'mpv_create' -or $names -notcontains 'mpv_initialize') { throw 'Missing mpv API exports' }
@('LIBRARY libmpv-2.dll', 'EXPORTS') + $names | Set-Content "$dest/mpv.def" -Encoding ascii
& lib /nologo "/def:$dest/mpv.def" /machine:x64 "/out:$dest/mpv.lib"
if ($LASTEXITCODE -ne 0 -or !(Test-Path "$dest/mpv.lib")) { throw 'MSVC import library generation failed' }
# The upstream mpv DLL also imports the Vulkan loader, which is not present
# on clean Windows machines without a vendor GPU driver. Bundle it locally.
$vulkanArchive = Join-Path $dest 'vulkan.zip'
if (!(Test-Path $vulkanArchive)) { Invoke-WebRequest $pin.vulkan.url -OutFile $vulkanArchive }
if ((Get-FileHash $vulkanArchive -Algorithm SHA256).Hash.ToLowerInvariant() -ne $pin.vulkan.sha256) {
    throw 'Vulkan checksum mismatch; refusing to extract downloaded content'
}
Expand-Archive $vulkanArchive -DestinationPath "$dest/vulkan" -Force
$vulkanRoot = Join-Path "$dest/vulkan" $pin.vulkan.directory
Copy-Item "$vulkanRoot/x64/vulkan-1.dll" "$dest/vulkan-1.dll"
Copy-Item "$vulkanRoot/VulkanRT-License.txt" "$dest/VulkanRT-License.txt"
$imports = & dumpbin /nologo /dependents $dll "$dest/vulkan-1.dll"
if ($LASTEXITCODE -ne 0) { throw 'Cannot inspect libmpv imports' }
foreach ($line in $imports) {
    if ($line -match '^\s+([\w.-]+\.dll)\s*$') {
        $dependency = $Matches[1]
        if (!(Test-Path "$env:SystemRoot/System32/$dependency") -and !(Test-Path "$dest/$dependency") -and $dependency -notmatch '^(api|ext)-ms-') {
            throw "Unbundled libmpv dependency: $dependency"
        }
    }
}
$imports | Set-Content "$dest/dependencies.txt"
Copy-Item "$PSScriptRoot/windows-mpv.json" "$dest/provenance.json"
Copy-Item "$root/docs/windows-preview-notice.txt" "$dest/NOTICE.txt"
Copy-Item "$root/LICENSE" "$dest/VELO-LICENSE.txt"
$env:VELO_MPV_LIB_DIR = $dest
$env:PATH = "$dest;$env:PATH"
if ($env:GITHUB_ENV) {
    "VELO_MPV_LIB_DIR=$dest" | Out-File $env:GITHUB_ENV -Append -Encoding utf8
    $dest | Out-File $env:GITHUB_PATH -Append -Encoding utf8
}
