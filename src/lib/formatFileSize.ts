export function formatFileSize(bytes: number, decimals = 1): string {
	if (bytes === 0) return '0 B'

	const units = ['B', 'KB', 'MB', 'GB']
	const i = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1)
	const size = bytes / Math.pow(1024, i)

	return `${size.toFixed(i === 0 ? 0 : decimals)} ${units[i]}`
}
