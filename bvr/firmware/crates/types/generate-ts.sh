#!/bin/bash
# Generate TypeScript types from Rust types crate
# Usage: ./generate-ts.sh [output-dir]
#
# Default output: depot/console/src/lib/generated/

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FIRMWARE_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"
DEFAULT_OUTPUT="$FIRMWARE_DIR/../../depot/console/src/lib/generated"

OUTPUT_DIR="${1:-$DEFAULT_OUTPUT}"

echo "Generating TypeScript types..."
echo "  Rust crate: $SCRIPT_DIR"
echo "  Output dir: $OUTPUT_DIR"

# Run tests with ts feature to generate bindings
cd "$FIRMWARE_DIR"
cargo test -p types --features ts --quiet

# Copy generated files to output directory
mkdir -p "$OUTPUT_DIR"
cp "$SCRIPT_DIR/bindings/generated/"*.ts "$OUTPUT_DIR/"

echo "Generated files:"
ls -1 "$OUTPUT_DIR/"*.ts | xargs -I {} basename {}

echo ""
echo "Done! Types generated to: $OUTPUT_DIR"
