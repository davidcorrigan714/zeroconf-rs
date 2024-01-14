Set-PSDebug -Trace 1

cargo build --workspace --verbose

cd examples
cargo build --workspace --verbose

