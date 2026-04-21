param(
    [string]$LogPath
)

$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'

$installerUrl = 'https://sourceforge.net/projects/equalizerapo/files/latest/download'
$installerPath = Join-Path ([System.IO.Path]::GetTempPath()) ("EqualizerAPO-{0}.exe" -f ([System.Guid]::NewGuid().ToString('N')))
$installerDirectory = Split-Path -Parent $installerPath
$registryPath = 'HKLM:\SOFTWARE\EqualizerAPO'

function Write-InstallerLog {
    param([string]$Message)

    if ([string]::IsNullOrWhiteSpace($LogPath)) {
        return
    }

    $logDirectory = Split-Path -Parent $LogPath
    if (-not [string]::IsNullOrWhiteSpace($logDirectory) -and -not (Test-Path $logDirectory)) {
        New-Item -ItemType Directory -Path $logDirectory -Force | Out-Null
    }

    Add-Content -LiteralPath $LogPath -Value $Message -Encoding UTF8
}

function Test-WindowsExecutable {
    param([string]$Path)

    $stream = [System.IO.File]::OpenRead($Path)
    try {
        $buffer = New-Object byte[] 2
        if ($stream.Read($buffer, 0, 2) -ne 2) {
            return $false
        }

        return $buffer[0] -eq 0x4D -and $buffer[1] -eq 0x5A
    }
    finally {
        $stream.Dispose()
    }
}

function Resolve-SourceForgeReleaseUrl {
    param([string]$PageUrl)

    $response = Invoke-WebRequest -Uri $PageUrl
    $content = $response.Content

    $releaseUrl = $null
    $dataReleaseUrlMatch = [System.Text.RegularExpressions.Regex]::Match(
        $content,
        'data-release-url="([^"]+)"',
        [System.Text.RegularExpressions.RegexOptions]::IgnoreCase
    )

    if ($dataReleaseUrlMatch.Success) {
        $releaseUrl = $dataReleaseUrlMatch.Groups[1].Value
    } else {
        $metaRefreshMatch = [System.Text.RegularExpressions.Regex]::Match(
            $content,
            'http-equiv="refresh"\s+content="\d+;\s*url=([^"]+)"',
            [System.Text.RegularExpressions.RegexOptions]::IgnoreCase
        )

        if ($metaRefreshMatch.Success) {
            $releaseUrl = $metaRefreshMatch.Groups[1].Value
        }
    }

    if ([string]::IsNullOrWhiteSpace($releaseUrl)) {
        throw 'Failed to resolve the SourceForge release URL from the latest download page.'
    }

    return [System.Net.WebUtility]::HtmlDecode($releaseUrl)
}

try {
    Write-InstallerLog 'Resolving the latest Equalizer APO download URL...'
    $releaseUrl = Resolve-SourceForgeReleaseUrl -PageUrl $installerUrl
    Write-InstallerLog "Resolved SourceForge release URL: $releaseUrl"

    Write-InstallerLog 'Downloading the latest Equalizer APO installer...'
    Invoke-WebRequest -Uri $releaseUrl -OutFile $installerPath

    if (-not (Test-Path $installerPath)) {
        throw 'The Equalizer APO installer was not downloaded.'
    }

    $installerInfo = Get-Item -LiteralPath $installerPath
    Write-InstallerLog ("Downloaded installer size: {0} bytes." -f $installerInfo.Length)

    if ($installerInfo.Length -lt 1024 -or -not (Test-WindowsExecutable -Path $installerPath)) {
        throw 'The downloaded installer is not a valid Windows executable. SourceForge may have returned unexpected content.'
    }

    Write-InstallerLog 'Running the silent installer...'
    Set-Location -Path $installerDirectory
    $installerProcess = Start-Process -FilePath $installerPath -ArgumentList '/S' -WorkingDirectory $installerDirectory -Wait -PassThru
    if ($installerProcess.ExitCode -ne 0) {
        throw "Equalizer APO installer exited with code $($installerProcess.ExitCode)."
    }

    $installPath = $null
    for ($attempt = 0; $attempt -lt 20 -and [string]::IsNullOrWhiteSpace($installPath); $attempt++) {
        try {
            $installPath = (Get-ItemProperty -Path $registryPath -Name 'InstallPath' -ErrorAction Stop).InstallPath
        } catch {
            Start-Sleep -Milliseconds 500
        }
    }

    if ([string]::IsNullOrWhiteSpace($installPath)) {
        throw 'Equalizer APO InstallPath was not found in the registry after installation.'
    }

    $deviceSelector = Join-Path $installPath 'DeviceSelector.exe'
    if (-not (Test-Path $deviceSelector)) {
        throw "DeviceSelector.exe was not found at '$deviceSelector'."
    }

    Write-InstallerLog "Launching Device Selector from '$deviceSelector'."
    Start-Process -FilePath $deviceSelector -WorkingDirectory $installPath
    Write-InstallerLog 'Device Selector launched successfully.'
}
catch {
    Write-InstallerLog "ERROR: $($_.Exception.Message)"
    throw
}
finally {
    if (Test-Path $installerPath) {
        Remove-Item $installerPath -Force -ErrorAction SilentlyContinue
    }
}
