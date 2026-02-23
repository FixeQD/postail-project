import type { ParsedAddress } from '@/types/mail'

const ADDRESS_REGEX = /^(.*?)\s*<(.+?)>$/

export function parseAddress(raw: string): ParsedAddress {
	const trimmed = raw.trim()
	const match = trimmed.match(ADDRESS_REGEX)

	if (match) {
		const name = match[1].replace(/^["']|["']$/g, '').trim()
		return {
			name: name || match[2],
			email: match[2],
		}
	}

	// bare email or weird format
	return { name: trimmed, email: trimmed }
}

export function parseAddresses(addresses: string[]): ParsedAddress[] {
	return addresses.map(parseAddress)
}
