import React, { ReactNode, useMemo } from 'react'
import { withScale } from '../../composables'
import { TextChild } from './child'

interface Props {
  h1?: boolean
  h2?: boolean
  h3?: boolean
  h4?: boolean
  h5?: boolean
  h6?: boolean
  p?: boolean
  b?: boolean
  small?: boolean
  i?: boolean
  span?: boolean
  del?: boolean
  em?: boolean
  blockquote?: boolean
  className?: string
}

type ElementMap = { [k in keyof JSX.IntrinsicElements]?: boolean }

type NativeAttrs = Omit<React.HTMLAttributes<unknown>, keyof Props>
export type TextProps = Props & NativeAttrs

type TextRenderableElements = Array<keyof JSX.IntrinsicElements>

const getModifierChild = (tags: TextRenderableElements, children: ReactNode) => {
  if (!tags.length) { return children }
  const nextTag = tags.slice(1, tags.length)
  return <TextChild tag={tags[0]}>{getModifierChild(nextTag, children)}</TextChild>
}

function TextComponent({
  h1,
  h2,
  h3,
  h4,
  h5,
  h6,
  p,
  b,
  small,
  i,
  span,
  del,
  em,
  blockquote,
  children,
  className,
  ...props
}: React.PropsWithChildren<TextProps>) {
  const elements: ElementMap = { h1, h2, h3, h4, h5, h6, p, blockquote }
  const inlineElements: ElementMap = { span, small, b, em, i, del }
  const names = Object.keys(elements).filter(
    (name: string) => elements[name as keyof JSX.IntrinsicElements]
  ) as TextRenderableElements
  const inlineNames = Object.keys(inlineElements).filter(
    (name: string) => inlineElements[name as keyof JSX.IntrinsicElements]
  ) as TextRenderableElements

  /**
   *  Render element "p" only if no element is found.
   *  If there is only one modifier, just rendered one modifier element
   *  e.g.
   *    <Text /> => <p />
   *    <Text em /> => <em />
   *    <Text p em /> => <p><em>children</em></p>
   */

  const tag = useMemo(() => {
    if (names[0]) { return names[0] }
    if (inlineNames[0]) { return inlineNames[0] }
    return 'p' as keyof JSX.IntrinsicElements
  }, [names, inlineNames])

  const renderableChildElements = inlineNames.filter(
    (name: keyof JSX.IntrinsicElements) => name !== tag
  ) as TextRenderableElements

  const modifers = useMemo(() => {
    if (!renderableChildElements.length) { return children }
    return getModifierChild(renderableChildElements, children)
  }, [renderableChildElements, children])

  return (
    <TextChild className={className} tag={tag} {...props}>
      {modifers}
    </TextChild>
  )
}

export const Text = withScale(TextComponent)
