package dev.sms.wallet

/**
 * Minimal Solana wire-format helpers: find the message bytes a signer signs and put a signature
 * in the right slot. Supports legacy and v0 transactions.
 *
 * Layout: compact-u16 signature count, 64-byte signatures, then the message:
 * [0x80 | version] (v0 only), header (3 bytes), compact-u16 account count, 32-byte keys, ...
 */
object SolanaTx {
    const val SIGNATURE_LEN = 64
    const val PUBLIC_KEY_LEN = 32

    class Parsed(
        /** Offset of the first signature slot. */
        val signaturesOffset: Int,
        val numSignatures: Int,
        /** Offset where the message (the signed bytes) starts. */
        val messageOffset: Int,
        /** Public keys of the required signers, in signature-slot order. */
        val signers: List<ByteArray>,
    )

    fun parse(tx: ByteArray): Parsed {
        require(tx.isNotEmpty()) { "empty transaction" }
        val (numSignatures, lenBytes) = readCompactU16(tx, 0)
        val messageOffset = lenBytes + SIGNATURE_LEN * numSignatures
        require(messageOffset < tx.size) { "signatures extend beyond the transaction" }
        val prefix = tx[messageOffset].toInt() and 0xff
        val versioned = prefix and 0x80 != 0
        if (versioned) require(prefix and 0x7f == 0) { "unsupported transaction version ${prefix and 0x7f}" }
        val headerOffset = messageOffset + if (versioned) 1 else 0
        require(headerOffset + 3 <= tx.size) { "truncated message header" }
        val numRequired = tx[headerOffset].toInt() and 0xff
        require(numRequired == numSignatures) { "signature count does not match the message header" }
        val (numAccounts, accLenBytes) = readCompactU16(tx, headerOffset + 3)
        val keysOffset = headerOffset + 3 + accLenBytes
        require(numAccounts >= numRequired && keysOffset + numAccounts * PUBLIC_KEY_LEN <= tx.size) { "bad account keys" }
        val signers = (0 until numRequired).map { i ->
            tx.copyOfRange(keysOffset + i * PUBLIC_KEY_LEN, keysOffset + (i + 1) * PUBLIC_KEY_LEN)
        }
        return Parsed(lenBytes, numSignatures, messageOffset, signers)
    }

    fun message(tx: ByteArray, parsed: Parsed = parse(tx)): ByteArray = tx.copyOfRange(parsed.messageOffset, tx.size)

    fun signerIndex(parsed: Parsed, publicKey: ByteArray): Int = parsed.signers.indexOfFirst { it.contentEquals(publicKey) }

    /** Returns a copy of [tx] with [signature] in [publicKey]'s slot. */
    fun withSignature(tx: ByteArray, publicKey: ByteArray, signature: ByteArray): ByteArray {
        val parsed = parse(tx)
        val index = signerIndex(parsed, publicKey)
        require(index >= 0) { "transaction does not need a signature from this account" }
        require(signature.size == SIGNATURE_LEN) { "bad signature length" }
        return tx.clone().also { System.arraycopy(signature, 0, it, parsed.signaturesOffset + index * SIGNATURE_LEN, SIGNATURE_LEN) }
    }

    private fun readCompactU16(b: ByteArray, offset: Int): Pair<Int, Int> {
        var value = 0
        var size = 0
        while (true) {
            require(offset + size < b.size) { "truncated compact-u16" }
            val byte = b[offset + size].toInt() and 0xff
            value = value or ((byte and 0x7f) shl (7 * size))
            size++
            if (byte and 0x80 == 0) break
            require(size < 3) { "compact-u16 too long" }
        }
        return value to size
    }
}

/** Base58 (Bitcoin alphabet), for showing addresses and signatures. */
object Base58 {
    private const val ALPHABET = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz"

    fun encode(input: ByteArray): String {
        if (input.isEmpty()) return ""
        val zeros = input.takeWhile { it == 0.toByte() }.size
        val digits = mutableListOf<Int>()
        for (b in input) {
            var carry = b.toInt() and 0xff
            for (i in digits.indices) {
                carry += digits[i] shl 8
                digits[i] = carry % 58
                carry /= 58
            }
            while (carry > 0) {
                digits.add(carry % 58)
                carry /= 58
            }
        }
        return "1".repeat(zeros) + digits.reversed().joinToString("") { ALPHABET[it].toString() }
    }
}
