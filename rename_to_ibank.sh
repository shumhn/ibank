#!/bin/bash
# Script to rename blackjack to ibank throughout the codebase

echo "🔄 Renaming project from 'blackjack' to 'ibank'..."

# 1. Rename the programs directory
if [ -d "programs/blackjack" ]; then
    mv programs/blackjack programs/ibank
    echo "✅ Renamed programs/blackjack to programs/ibank"
else
    echo "⚠️  programs/blackjack already renamed or doesn't exist"
fi

# 2. Update Anchor.toml
sed -i '' 's/blackjack = /ibank = /g' Anchor.toml
echo "✅ Updated Anchor.toml"

# 3. Update programs/ibank/Cargo.toml
sed -i '' 's/name = "blackjack"/name = "ibank"/g' programs/ibank/Cargo.toml
echo "✅ Updated programs/ibank/Cargo.toml"

# 4. Update programs/ibank/src/lib.rs (module name)
sed -i '' 's/pub mod blackjack {/pub mod ibank {/g' programs/ibank/src/lib.rs
echo "✅ Updated module name in lib.rs"

# 5. Update test imports
sed -i '' 's/{ Blackjack }/{ Ibank }/g' tests/banking_demo.ts
sed -i '' 's/Program<Blackjack>/Program<Ibank>/g' tests/banking_demo.ts
sed -i '' 's/"..\/target\/types\/blackjack"/"..\/target\/types\/ibank"/g' tests/banking_demo.ts
echo "✅ Updated test file imports"

# 6. Update Cargo workspace member if needed
sed -i '' 's/programs\/blackjack/programs\/ibank/g' Cargo.toml
echo "✅ Updated Cargo.toml workspace"

# 7. Clean old build artifacts
echo "🧹 Cleaning old build artifacts..."
rm -rf target/deploy/blackjack*
rm -rf target/idl/blackjack*
rm -rf target/types/blackjack*

echo ""
echo "✅ Renaming complete!"
echo ""
echo "Next steps:"
echo "1. Run: anchor build"
echo "2. Run: arcium build"
echo "3. Run: arcium test"
echo ""