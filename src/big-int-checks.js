const max_signed = 2n ** (64n - 1n) - 1n;
const min_signed = -1n * 2n ** (64n - 1n);
const max_unsigned = 2n ** 64n - 1n;
const min_unsigned = 0n;

export function to_i64(num) {
    if(num <= max_signed && num >= min_signed) {
        return BigInt(num);
    }
    return undefined;
}

export function to_u64(num) {
    if(num <= max_unsigned && num >= min_unsigned) {
        return BigInt(num);
    }
    return undefined;
}