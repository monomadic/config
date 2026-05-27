param(
    [string]$Destination = "$env:LOCALAPPDATA\Programs\leaf"
)

$ErrorActionPreference = "Stop"

$Repo = "RivoLink/leaf"
$AssetName = "leaf-windows-x86_64.exe"

function Write-Info {
    param([string]$Message)
    Write-Host $Message
}

function Ensure-InstallDir {
    param([string]$Dir)

    New-Item -ItemType Directory -Force -Path $Dir | Out-Null
}

function Get-LatestTag {
    param([string]$Repo)

    $release = Invoke-RestMethod "https://api.github.com/repos/$Repo/releases/latest"
    if (-not $release.tag_name) {
        throw "Unable to resolve latest release tag for $Repo"
    }
    return $release.tag_name
}

function Get-DownloadUrl {
    param(
        [string]$Repo,
        [string]$Tag,
        [string]$Asset
    )

    return "https://github.com/$Repo/releases/download/$Tag/$Asset"
}

function Download-Binary {
    param(
        [string]$Url
    )

    $tempFile = [System.IO.Path]::GetTempFileName()
    try {
        Invoke-WebRequest -UseBasicParsing -Uri $Url -OutFile $tempFile
        return $tempFile
    } catch {
        Remove-Item -Path $tempFile -Force -ErrorAction SilentlyContinue
        throw
    }
}

function Verify-Checksum {
    param(
        [string]$File,
        [string]$ChecksumsUrl,
        [string]$AssetName
    )
    $checksums = Invoke-RestMethod -Uri $ChecksumsUrl
    $escapedName = [regex]::Escape($AssetName)
    $line = ($checksums -split "\r?\n") |
        Where-Object { $_ -match "\s${escapedName}$" } |
        Select-Object -First 1
    if (-not $line) {
        throw "Checksum not found for $AssetName"
    }
    $expected = ($line -split '\s+')[0]
    $actual = (Get-FileHash -Path $File -Algorithm SHA256).Hash.ToLower()
    if ($actual -ne $expected) {
        throw "Checksum mismatch!`nExpected: $expected`nGot:      $actual"
    }
}

function Add-ToUserPath {
    param([string]$Dir)

    $currentPath = [Environment]::GetEnvironmentVariable('Path', 'User')
    $pathParts = @($currentPath -split ';' | Where-Object { $_ -ne '' })

    if ($Dir -notin $pathParts) {
        $pathParts += $Dir
        [Environment]::SetEnvironmentVariable('Path', ($pathParts -join ';'), 'User')
        if ($env:Path) {
            $env:Path = "$Dir;$env:Path"
        } else {
            $env:Path = $Dir
        }
        Write-Info "Added $Dir to your user PATH"
        Write-Info "PATH updated for current session"
    } else {
        Write-Info "$Dir is already in your user PATH"
    }
}

$destinationDir = $Destination
$destinationBin = Join-Path $destinationDir "leaf.exe"

Ensure-InstallDir -Dir $destinationDir

$currentVersion = $null
if (Test-Path $destinationBin) {
    try { $currentVersion = ((& $destinationBin --version 2>$null) -split ' ')[1] } catch {}
}

if ($currentVersion) {
    Write-Info "Updating leaf..."
} else {
    Write-Info "Installing leaf..."
}

$tagName = Get-LatestTag -Repo $Repo
$downloadUrl = Get-DownloadUrl -Repo $Repo -Tag $tagName -Asset $AssetName

$tempFile = Download-Binary -Url $downloadUrl
try {
    $checksumsUrl = Get-DownloadUrl -Repo $Repo -Tag $tagName -Asset "checksums.txt"
    Verify-Checksum -File $tempFile -ChecksumsUrl $checksumsUrl -AssetName $AssetName
    Copy-Item -Path $tempFile -Destination $destinationBin -Force
} finally {
    Remove-Item -Path $tempFile -Force -ErrorAction SilentlyContinue
}
Add-ToUserPath -Dir $destinationDir

$newVersion = $tagName -replace '^v', ''
if ($currentVersion) {
    Write-Info "leaf updated from $currentVersion to $newVersion"
} else {
    Write-Info "leaf $newVersion installed"
}
