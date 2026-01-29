import {
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
	height?: number
	key?: NodeKey
	maxWidth?: number
	src: string
	width?: number
}

export type SerializedImageNode = Spread<
	{
		altText: string
		height?: number
		maxWidth?: number
		src: string
		width?: number
	},
	SerializedLexicalNode
>

export class ImageNode extends DecoratorNode<React.ReactNode> {
	__src: string
	__altText: string
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
			node.__width,
			node.__height,
			node.__maxWidth,
			node.__key
		)
	}

	static importJSON(serializedNode: SerializedImageNode): ImageNode {
		const { altText, height, width, maxWidth, src } = serializedNode
		const node = $createImageNode({
			altText,
			height,
			maxWidth,
			src,
			width,
		})
		return node
	}

	exportDOM(): DOMExportOutput {
		const element = document.createElement('img')
		element.setAttribute('src', this.__src)
		element.setAttribute('alt', this.__altText)
		if (this.__width) element.setAttribute('width', this.__width.toString())
		if (this.__height) element.setAttribute('height', this.__height.toString())
		return { element }
	}

	constructor(
		src: string,
		altText: string,
		width?: number,
		height?: number,
		maxWidth?: number,
		key?: NodeKey
	) {
		super(key)
		this.__src = src
		this.__altText = altText
		this.__width = width
		this.__height = height
		this.__maxWidth = maxWidth
	}

	exportJSON(): SerializedImageNode {
		return {
			altText: this.getAltText(),
			height: this.__height,
			maxWidth: this.__maxWidth,
			src: this.getSrc(),
			type: 'image',
			version: 1,
			width: this.__width,
		}
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
			<img
				src={this.__src}
				alt={this.__altText}
				style={{
					display: 'block',
					maxWidth: '100%',
					width: this.__width,
					height: this.__height,
					borderRadius: '8px',
					margin: '8px 0',
				}}
			/>
		)
	}
}

export function $createImageNode({
	altText,
	height,
	maxWidth = 500,
	src,
	width,
	key,
}: ImagePayload): ImageNode {
	return new ImageNode(src, altText, width, height, maxWidth, key)
}

export function $isImageNode(node: LexicalNode | null | undefined): node is ImageNode {
	return node instanceof ImageNode
}
