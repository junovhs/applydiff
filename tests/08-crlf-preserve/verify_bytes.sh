#!/bin/bash
# Binary verification that line endings are preserved

echo "=== Byte-Level Line Ending Verification ==="

# Function to show hex dump of line endings
show_endings() {
    local file=$1
    echo "File: $file"
    # Show hex bytes, highlight 0d (CR) and 0a (LF)
    hexdump -C "$file" | grep -E "0d 0a|0a" | head -5
    echo ""
}

# Check CRLF file
echo "1. Windows.txt (should be all CRLF = 0d 0a):"
if hexdump -ve '1/1 "%.2x "' after/windows.txt | grep -q "0d 0a.*0d 0a.*0d 0a"; then
    echo "   ✓ CRLF preserved"
else
    echo "   ✗ CRLF NOT preserved - FAIL"
    exit 1
fi

# Check LF file  
echo "2. Unix.txt (should be all LF = 0a, NO 0d):"
if hexdump -ve '1/1 "%.2x "' after/unix.txt | grep -q "0a" && \
   ! hexdump -ve '1/1 "%.2x "' after/unix.txt | grep -q "0d"; then
    echo "   ✓ LF preserved, no CRLF contamination"
else
    echo "   ✗ LF NOT preserved or CRLF leaked - FAIL"
    exit 1
fi

# Check mixed file (line 1 and 3 should be CRLF, line 2 should be LF)
echo "3. Mixed.txt (line 1: CRLF, line 2: LF, line 3: CRLF):"
hex_mixed=$(hexdump -ve '1/1 "%.2x "' after/mixed.txt)
# Count CRLF sequences (0d 0a)
crlf_count=$(echo "$hex_mixed" | grep -o "0d 0a" | wc -l)
# Count total newlines (0a)
lf_count=$(echo "$hex_mixed" | grep -o "0a" | wc -l)

if [ "$crlf_count" -eq 2 ] && [ "$lf_count" -eq 3 ]; then
    echo "   ✓ Mixed endings preserved (2 CRLF, 1 solo LF)"
else
    echo "   ✗ Mixed endings NOT preserved (found $crlf_count CRLF, $lf_count total LF) - FAIL"
    exit 1
fi

# Check harmonization (should add CRLF to match internal pattern)
echo "4. No_trailing.txt (should add CRLF to match file's pattern):"
if hexdump -ve '1/1 "%.2x "' after/no_trailing.txt | tail -c 10 | grep -q "0d 0a"; then
    echo "   ✓ Harmonization added CRLF (not LF)"
else
    echo "   ✗ Harmonization failed - FAIL"
    exit 1
fi

echo ""
echo "=== Binary Comparison with Expected ==="

# Byte-for-byte comparison with expected files
for file in windows.txt unix.txt mixed.txt no_trailing.txt; do
    if cmp -s "after/$file" "../after/$file" 2>/dev/null; then
        echo "✓ $file matches expected byte-for-byte"
    else
        echo "✗ $file differs from expected - FAIL"
        echo "  Diff:"
        diff -u <(hexdump -C "after/$file") <(hexdump -C "../after/$file") | head -20
        exit 1
    fi
done

echo ""
echo "=== SHA256 Hash Verification ==="
cd after
sha256sum windows.txt unix.txt mixed.txt no_trailing.txt
cd ..

echo ""
echo "✓✓✓ ALL BYTE-LEVEL CHECKS PASSED ✓✓✓"
