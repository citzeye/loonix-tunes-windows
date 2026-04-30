Add-Type -AssemblyName System.Drawing
$png = [System.Drawing.Image]::FromFile('C:\loonix-tunes-windows\assets\LoonixTunes.png')
$icon = [System.Drawing.Icon]::FromHandle($png.GetHicon())
$stream = [System.IO.File]::Create('C:\loonix-tunes-windows\packaging\windows\icon.ico')
$icon.Save($stream)
$stream.Close()
$png.Dispose()
Write-Host 'Icon created successfully at packaging\windows\icon.ico'