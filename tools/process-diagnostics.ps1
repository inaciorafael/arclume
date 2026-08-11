param(
    [Parameter(Mandatory = $true)]
    [int]$ProcessId,
    [ValidateRange(1, 10000)]
    [int]$Samples = 200,
    [ValidateRange(10, 60000)]
    [int]$IntervalMilliseconds = 50
)

$workingSet = [System.Collections.Generic.List[double]]::new()
$privateMemory = [System.Collections.Generic.List[double]]::new()

for ($index = 0; $index -lt $Samples; $index++) {
    $process = Get-Process -Id $ProcessId -ErrorAction Stop
    $workingSet.Add([double]$process.WorkingSet64)
    $privateMemory.Add([double]$process.PrivateMemorySize64)
    if ($index -lt ($Samples - 1)) {
        Start-Sleep -Milliseconds $IntervalMilliseconds
    }
}

function Get-Percentile {
    param([double[]]$Values, [double]$Percentile)
    $sorted = $Values | Sort-Object
    $position = [Math]::Max(0, [Math]::Ceiling($sorted.Count * $Percentile) - 1)
    return [long]$sorted[$position]
}

[ordered]@{
    capturedAt = [DateTimeOffset]::UtcNow.ToString('o')
    platform = 'windows'
    processId = $ProcessId
    samples = $Samples
    intervalMilliseconds = $IntervalMilliseconds
    workingSetBytes = [ordered]@{
        p50 = Get-Percentile $workingSet 0.50
        p95 = Get-Percentile $workingSet 0.95
        p99 = Get-Percentile $workingSet 0.99
        peak = [long](($workingSet | Measure-Object -Maximum).Maximum)
    }
    privateMemoryBytes = [ordered]@{
        p50 = Get-Percentile $privateMemory 0.50
        p95 = Get-Percentile $privateMemory 0.95
        p99 = Get-Percentile $privateMemory 0.99
        peak = [long](($privateMemory | Measure-Object -Maximum).Maximum)
    }
} | ConvertTo-Json -Depth 4
