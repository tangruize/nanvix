#!/bin/bash
# Check all split modules against their source files

cd /home/ubuntu/nanvix

echo "=============================================="
echo "Checking all split modules"
echo "=============================================="
echo ""

# Define mappings: src_path:exec_path
MAPPINGS=(
    # libs
    "src/libs/error/src/lib.rs:verus/split/libs/error/lib.rs"
    "src/libs/raw-array/src/lib.rs:verus/split/libs/raw_array/lib.rs"
    "src/libs/bitmap/src/lib.rs:verus/split/libs/bitmap/lib.rs"
    "src/libs/slab/src/lib.rs:verus/split/libs/slab/lib.rs"
    
    # kernel/hal/mem/types/address
    # Note: frame_address doesn't have a direct src equivalent
    
    # kernel/mm/phys
    "src/kernel/src/mm/phys/frame.rs:verus/split/kernel/mm/phys/frame.rs"
    "src/kernel/src/mm/phys/kpool.rs:verus/split/kernel/mm/phys/kpool.rs"
    "src/kernel/src/mm/phys/upool.rs:verus/split/kernel/mm/phys/upool.rs"
    "src/kernel/src/mm/phys/manager.rs:verus/split/kernel/mm/phys/manager.rs"
    
    # kernel/mm
    "src/kernel/src/mm/kheap.rs:verus/split/kernel/mm/kheap.rs"
    "src/kernel/src/mm/kstack.rs:verus/split/kernel/mm/kstack.rs"
    "src/kernel/src/mm/ustack.rs:verus/split/kernel/mm/ustack.rs"
    "src/kernel/src/mm/kredzone.rs:verus/split/kernel/mm/kredzone.rs"
    
    # kernel/mm/virt
    "src/kernel/src/mm/virt/kpage.rs:verus/split/kernel/mm/virt/kpage.rs"
    "src/kernel/src/mm/virt/vmem.rs:verus/split/kernel/mm/virt/vmem.rs"
)

PASS=0
FAIL=0

for mapping in "${MAPPINGS[@]}"; do
    src="${mapping%%:*}"
    exec="${mapping##*:}"
    
    echo "----------------------------------------------"
    echo "Checking: $exec"
    echo "----------------------------------------------"
    
    if [ ! -f "$src" ]; then
        echo "WARNING: Source file not found: $src"
        echo ""
        continue
    fi
    
    if [ ! -f "$exec" ]; then
        echo "ERROR: Exec file not found: $exec"
        FAIL=$((FAIL + 1))
        echo ""
        continue
    fi
    
    python3 verus/split/scripts/compare_split.py "$src" "$exec"
    
    if [ $? -eq 0 ]; then
        PASS=$((PASS + 1))
    else
        FAIL=$((FAIL + 1))
    fi
    echo ""
done

echo "=============================================="
echo "Summary: $PASS passed, $FAIL failed"
echo "=============================================="
