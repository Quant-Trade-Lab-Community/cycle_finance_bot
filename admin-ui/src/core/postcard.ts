export function decodeVarintBig(buffer: Uint8Array, offset: { val: number }): bigint {
    let result = 0n;
    let shift = 0n;
    while (offset.val < buffer.length) {
        let byte = BigInt(buffer[offset.val++]);
        result |= (byte & 0x7fn) << shift;
        if ((byte & 0x80n) === 0n) break;
        shift += 7n;
    }
    return result;
}

export function decodeZigZagBig(buffer: Uint8Array, offset: { val: number }): bigint {
    let val = decodeVarintBig(buffer, offset);
    return (val >> 1n) ^ -(val & 1n);
}

export function decode(buffer: Uint8Array): any {
    let offset = { val: 0 };
    const p99_latency_ns = Number(decodeVarintBig(buffer, offset));
    const pnl = Number(decodeZigZagBig(buffer, offset));
    const free_balance = Number(decodeVarintBig(buffer, offset));
    const ring_buffer_usage = buffer[offset.val++];
    
    return {
        p99_latency_ns,
        pnl,
        free_balance,
        ring_buffer_usage
    };
}
