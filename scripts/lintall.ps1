Set-PSDebug -Trace 1

cargo clippy -- -D warnings
if ($LASTEXITCODE -ne 0) { Exit $LASTEXITCODE }
cargo clippy --all-targets --all-features -- -D warnings
if ($LASTEXITCODE -ne 0) { Exit $LASTEXITCODE }

cd examples
cargo clippy -- -D warnings
if ($LASTEXITCODE -ne 0) { Exit $LASTEXITCODE }
cargo clippy --all-targets --all-features -- -D warnings
if ($LASTEXITCODE -ne 0) { Exit $LASTEXITCODE }
