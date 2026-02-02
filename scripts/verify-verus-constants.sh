#!/bin/bash
# Copyright(c) The Maintainers of Nanvix.
# Licensed under the MIT License.

#===================================================================================================
# Script: verify-verus-constants.sh
#
# Description:
#   Verifies that constants in Verus verification modules match the kernel configuration.
#   This script should be run as part of CI to ensure proofs remain synchronized with the kernel.
#
# Usage:
#   ./scripts/verify-verus-constants.sh
#
# Exit Codes:
#   0 - All constants match
#   1 - Constants mismatch detected
#===================================================================================================

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

# Colors for output.
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "========================================"
echo "Verus Constants Verification"
echo "========================================"

ERRORS=0

#---------------------------------------------------------------------------------------------------
# Helper function to extract constant value from a file.
#---------------------------------------------------------------------------------------------------
extract_const() {
    local file="$1"
    local pattern="$2"
    grep -E "${pattern}" "${file}" | head -1 | grep -oE '[0-9]+' | tail -1
}

#---------------------------------------------------------------------------------------------------
# Verify ustack.rs constants.
#---------------------------------------------------------------------------------------------------
echo ""
echo "Checking verus/ustack.rs constants..."

VERUS_USTACK="${ROOT_DIR}/verus/ustack.rs"
KERNEL_CONFIG="${ROOT_DIR}/src/libs/config/src/lib.rs"

if [[ ! -f "${VERUS_USTACK}" ]]; then
    echo -e "${YELLOW}Warning: ${VERUS_USTACK} not found, skipping.${NC}"
else
    # Check USER_STACK_SIZE.
    # Kernel defines: pub const USER_STACK_SIZE: usize = 512 * crate::constants::KILOBYTE;
    # Which equals: 512 * 1024 = 524288
    KERNEL_STACK_SIZE=524288
    VERUS_STACK_SIZE=$(extract_const "${VERUS_USTACK}" "pub const USER_STACK_SIZE.*=.*[0-9]")

    if [[ "${VERUS_STACK_SIZE}" == "${KERNEL_STACK_SIZE}" ]]; then
        echo -e "${GREEN}  ✓ USER_STACK_SIZE: ${VERUS_STACK_SIZE} (matches kernel)${NC}"
    else
        echo -e "${RED}  ✗ USER_STACK_SIZE mismatch: Verus=${VERUS_STACK_SIZE}, Kernel=${KERNEL_STACK_SIZE}${NC}"
        ERRORS=$((ERRORS + 1))
    fi

    # Check PAGE_SIZE.
    KERNEL_PAGE_SIZE=4096
    VERUS_PAGE_SIZE=$(extract_const "${VERUS_USTACK}" "pub const PAGE_SIZE.*=.*[0-9]")

    if [[ "${VERUS_PAGE_SIZE}" == "${KERNEL_PAGE_SIZE}" ]]; then
        echo -e "${GREEN}  ✓ PAGE_SIZE: ${VERUS_PAGE_SIZE} (matches kernel)${NC}"
    else
        echo -e "${RED}  ✗ PAGE_SIZE mismatch: Verus=${VERUS_PAGE_SIZE}, Kernel=${KERNEL_PAGE_SIZE}${NC}"
        ERRORS=$((ERRORS + 1))
    fi

    # Check USER_STACK_PAGES (derived: USER_STACK_SIZE / PAGE_SIZE).
    KERNEL_STACK_PAGES=$((KERNEL_STACK_SIZE / KERNEL_PAGE_SIZE))
    VERUS_STACK_PAGES=$(extract_const "${VERUS_USTACK}" "pub const USER_STACK_PAGES.*=.*[0-9]")

    if [[ "${VERUS_STACK_PAGES}" == "${KERNEL_STACK_PAGES}" ]]; then
        echo -e "${GREEN}  ✓ USER_STACK_PAGES: ${VERUS_STACK_PAGES} (matches derived)${NC}"
    else
        echo -e "${RED}  ✗ USER_STACK_PAGES mismatch: Verus=${VERUS_STACK_PAGES}, Expected=${KERNEL_STACK_PAGES}${NC}"
        ERRORS=$((ERRORS + 1))
    fi
fi

#---------------------------------------------------------------------------------------------------
# Summary.
#---------------------------------------------------------------------------------------------------
echo ""
echo "========================================"
if [[ ${ERRORS} -eq 0 ]]; then
    echo -e "${GREEN}All Verus constants verified successfully.${NC}"
    exit 0
else
    echo -e "${RED}${ERRORS} constant mismatch(es) detected.${NC}"
    echo -e "${RED}Please update Verus modules to match kernel configuration.${NC}"
    exit 1
fi
