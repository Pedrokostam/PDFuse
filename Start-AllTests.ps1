[CmdletBinding()]
param(
   [string]
   $Path = "."
)
try
{
   Push-Location $Path
   $json = cargo metadata --no-deps --format-version 1 | ConvertFrom-Json
   $tests_failed = [System.Collections.ArrayList]::new()
   $tests_succeeded = [System.Collections.ArrayList]::new()
   foreach($package in $json.packages)
   {
      $output = cargo test -p $package.name 2>&1
      $normal_tests = $false
      $doc_tests = $false
      foreach($line in $output)
      {
         if([string]::IsNullOrWhiteSpace($line))
         {
            continue
         }
         if($line -match "Running\s+\d+\s+test")
         {
            $normal_tests = $true
            $doc_tests=$false
            continue
         }
         if($line -match "doc-test")
         {
            $normal_tests = $false
            $doc_tests = $true
            continue
         }
         if($line -match 'test result:')
         {
            $normal_tests=$false
            continue
         }
         if($normal_tests)
         {
            $groupMatch = [regex]::match($line,"test\s(.*)\s+.*\s+(\w+)")
            if(-not $groupMatch.Success)
            {
               continue
            }
            $group = $groupMatch.Groups
            $name = $group[1].Value
            $result = $group[2].Value -eq "ok"
            if($result)
            {
               $null= $tests_succeeded.Add($name)
            } else
            {
               $null= $tests_failed.Add($name)
            }
         }
      }C:\Program Files\Git\usr\bin\sh.exe
   }
   $tests_failed | Sort-Object | ForEach-Object {} { Write-host $_ -ForegroundColor Red } { Write-host '' }
   $tests_succeeded | Sort-Object| ForEach-Object { Write-host  $_  -ForegroundColor Green }
} finally
{
   Pop-Location
}
