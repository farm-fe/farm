export interface IPluginOptions {
	/**
	 * Compression algorithm
	 * @default brotli
	 */
	algorithm?: "gzip" | "brotli" | "deflateRaw" | "deflate";
	/**
	 * Compression level
	 * @default 6
	 */
	level?: number;
	/**
	 * Compression threshold
	 *
	 * **NOTE**: This option is in bytes.
	 *
	 * @default 1024
	 */
	threshold?: number;
	/**
	 * Compression filter
	 *
	 * **NOTE**: This option is a regular expression string, not a regular expression object.
	 * Deserialization will cause a panic if use regex object. For example,
	 *
	 * @default '\\.(js|mjs|json|css|html)$'
	 */
	filter?: string;
	/**
	 * Delete original file
	 * @default false
	 */
	deleteOriginFile?: boolean;
}
