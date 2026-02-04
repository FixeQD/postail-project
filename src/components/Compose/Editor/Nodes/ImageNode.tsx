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

export function $isImageNode(node: LexicalNode | null | undefined): node is ImageNode {
	return node instanceof ImageNode
}
