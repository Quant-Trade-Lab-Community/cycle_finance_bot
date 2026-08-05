#!/bin/bash

# GDPR/KVKK Right to Erasure Simulation Script for Demir Yumruk 2.0
# Simulates physically wiping a user's data from ClickHouse.

set -e

USER_ID="client_9942"
SALT="sUp3rS3cr3tS4lt"

echo "=========================================="
echo " Starting GDPR Erasure Protocol Simulation"
echo "=========================================="

# 1. Masking the User ID to create the hash
USER_HASH=$(echo -n "${USER_ID}${SALT}" | sha3sum -a 256 | awk '{print $1}' || echo "mocked_sha3_hash_8a2b3c")
echo "[+] Target User Hash: $USER_HASH"

# 2. Simulate ClickHouse Mutation
echo "[+] Triggering ALTER TABLE ticks DELETE WHERE symbol_hash = '$USER_HASH'..."
sleep 1
echo "[+] ClickHouse mutation submitted."

# 3. Simulate verifying the physical erasure
echo "[+] Verifying erasure via Merkle Tree..."
sleep 1
echo "[+] Data physically scrubbed from disks and EC-12/4 replicas."

# 4. Log to Deletion Registry
echo "[+] Appending event to deletion_registry for 3-year compliance hold..."
echo "$(date -u +%Y-%m-%dT%H:%M:%SZ) | $USER_HASH | GDPR Erasure Processed" >> deletion_registry_mock.log

echo "=========================================="
echo " Erasure Protocol Complete."
echo "=========================================="
