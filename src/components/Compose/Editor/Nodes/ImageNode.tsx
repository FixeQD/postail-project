import {
	DOMConversionMap,
	DOMConversionOutput,
	DOMExportOutput,
	DecoratorNode,
	EditorConfig,
	LexicalNode,
	NodeKey,
	SerializedLexicalNode,
	Spread,
} from 'lexical'
import React from 'react'

export interface ImagePayload {
	altText: string
	attachmentId?: string
	cid?: string
	height?: number
	key?: NodeKey
	maxWidth?: number
	src: string
	width?: number
}

export type SerializedImageNode = Spread<
	{
		altText: string
		attachmentId?: string
		cid?: string
		height?: number
		maxWidth?: number
		src: string
		width?: number
	},
	SerializedLexicalNode
>

/**
 * Converts a DOM node into a Lexical DOM conversion output when the node is an HTMLImageElement.
 *
 * @param domNode - The DOM node to convert; if it's an `<img>` element its attributes are used to create the node.
 * @returns A `DOMConversionOutput` containing an `ImageNode` built from the image element's attributes, or `null` if `domNode` is not an `<img>` element.
 */
function convertImageElement(domNode: Node): null | DOMConversionOutput {
	if (domNode instanceof HTMLImageElement) {
		const { alt: altText, src, width, height } = domNode

		const attachmentId = domNode.getAttribute('data-attachment-id') || undefined
		const cid = domNode.getAttribute('data-cid') || undefined
		const node = $createImageNode({
			altText,
			attachmentId,
			cid,
			height,
			src,
			width,
		})
		return { node }
	}
	return null
}

/**
 * Renders an <img> element for the given source with responsive styling and optional dimensions.
 *
 * @param src - Image source URL
 * @param altText - Alternative text for the image
 * @param width - Optional width in pixels; if omitted, the image width is determined by layout
 * @param height - Optional height in pixels; if omitted, the image height is determined by layout
 * @returns A React element that displays the image
 */
function ImageComponent({
	src,
	altText,
	width,
	height,
}: {
	src: string
	altText: string
	width?: number
	height?: number
}) {
	return (
		<img
			src={src}
			alt={altText}
			style={{
				display: 'block',
				maxWidth: '100%',
				width: width || 'auto',
				height: height || 'auto',
				borderRadius: '8px',
				margin: '8px 0',
			}}
		/>
	)
}

export class ImageNode extends DecoratorNode<React.ReactNode> {
	__src: string
	__altText: string
	__attachmentId?: string
	__cid?: string
	__width?: number
	__height?: number
	__maxWidth?: number

	static getType(): string {
		return 'image'
	}

	static clone(node: ImageNode): ImageNode {
		return new ImageNode(
			node.__src,
			node.__altText,
			node.__attachmentId,
			node.__cid,
			node.__width,
			node.__height,
			node.__maxWidth,
			node.__key
		)
	}

	static importJSON(serializedNode: SerializedImageNode): ImageNode {
		const { altText, attachmentId, cid, height, width, maxWidth, src } = serializedNode
		const node = $createImageNode({
			altText,
			attachmentId,
			cid,
			height,
			maxWidth,
			src,
			width,
		})
		return node
	}

	static importDOM(): DOMConversionMap | null {
		return {
			img: () => ({
				conversion: convertImageElement,
				priority: 0,
			}),
		}
	}

	constructor(
		src: string,
		altText: string,
		attachmentId?: string,
		cid?: string,
		width?: number,
		height?: number,
		maxWidth?: number,
		key?: NodeKey
	) {
		super(key)
		this.__src = src
		this.__altText = altText
		this.__attachmentId = attachmentId
		this.__cid = cid
		this.__width = width
		this.__height = height
		this.__maxWidth = maxWidth
	}

	exportJSON(): SerializedImageNode {
		return {
			altText: this.getAltText(),
			attachmentId: this.__attachmentId,
			cid: this.__cid,
			height: this.__height || 0,
			maxWidth: this.__maxWidth || 500,
			src: this.getSrc(),
			type: 'image',
			version: 1,
			width: this.__width || 0,
		}
	}

	exportDOM(): DOMExportOutput {
		const element = document.createElement('img')
		element.setAttribute('src', this.__src)
		element.setAttribute('alt', this.__altText)
		if (this.__attachmentId) element.setAttribute('data-attachment-id', this.__attachmentId)
		if (this.__cid) element.setAttribute('data-cid', this.__cid)
		if (this.__width) element.setAttribute('width', this.__width.toString())
		if (this.__height) element.setAttribute('height', this.__height.toString())
		return { element }
	}

	getSrc(): string {
		return this.__src
	}

	getAltText(): string {
		return this.__altText
	}

	createDOM(config: EditorConfig): HTMLElement {
		const span = document.createElement('span')
		const theme = config.theme
		const className = theme.image
		if (className !== undefined) {
			span.className = className
		}
		return span
	}

	updateDOM(): false {
		return false
	}

	decorate(): React.ReactNode {
		return (
			<ImageComponent
				src={this.__src}
				altText={this.__altText}
				width={this.__width}
				height={this.__height}
			/>
		)
	}
}

/**
 * Create a new ImageNode from the provided image payload.
 *
 * @param altText - Alternative text for the image
 * @param attachmentId - Optional attachment identifier for persisted assets
 * @param cid - Optional content identifier (e.g., for external storage references)
 * @param height - Optional image height in pixels
 * @param maxWidth - Maximum display width in pixels (defaults to 500)
 * @param src - Image source URL
 * @param width - Optional image width in pixels
 * @param key - Optional node key to reuse an existing node identity
 * @returns A new ImageNode representing the supplied image data
 */
export function $createImageNode({
	altText,
	attachmentId,
	cid,
	height,
	maxWidth = 500,
	src,
	width,
	key,
}: ImagePayload): ImageNode {
	return new ImageNode(src, altText, attachmentId, cid, width, height, maxWidth, key)
}

/**
 * Determines whether a given Lexical node is an ImageNode.
 *
 * @param node - The node to test
 * @returns `true` if the node is an ImageNode, `false` otherwise
 */
export function $isImageNode(node: LexicalNode | null | undefined): node is ImageNode {
	return node instanceof ImageNode
}