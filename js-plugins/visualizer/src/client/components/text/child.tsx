import { inline } from '@stylex-extend/core'
import * as stylex from '@stylexjs/stylex'
import { clsx } from 'clsx'
import React, { useMemo } from 'react'
import { useScale } from '../../composables'

export interface Props {
  tag: keyof JSX.IntrinsicElements
  className?: string
}

type NativeAttrs = Omit<React.DetailsHTMLAttributes<unknown>, keyof Props>
export type TextChildProps = Props & NativeAttrs

function TextChild({
  children,
  tag,
  className: userClassName,
  ...props
}: React.PropsWithChildren<TextChildProps>) {
  const Component = tag
  const { SCALES, getScaleProps } = useScale()
  const font = getScaleProps('font')
  const mx = getScaleProps(['margin', 'marginLeft', 'marginRight', 'mx', 'ml', 'mr'])
  const my = getScaleProps(['margin', 'marginTop', 'marginBottom', 'my', 'mt', 'mb'])
  const px = getScaleProps(['padding', 'paddingLeft', 'paddingRight', 'pl', 'pr', 'px'])
  const py = getScaleProps(['padding', 'paddingTop', 'paddingBottom', 'pt', 'pb', 'py'])
  const classNames = useMemo<string>(() => {
    const keys = [
      { value: mx, className: 'mx' },
      { value: my, className: 'my' },
      { value: px, className: 'px' },
      { value: py, className: 'py' },
      { value: font, className: 'font' }
    ]
    const scaleClassNames = keys.reduce((pre, next) => {
      if (typeof next.value === 'undefined') { return pre }
      return `${pre} ${next.className}`
    }, '')
    return `${scaleClassNames} ${userClassName}`.trim()
  }, [mx, my, px, py, font, userClassName])

  const { className, style } = stylex.props(inline({
    width: SCALES.width(1, 'auto'),
    height: SCALES.height(1, 'auto'),
    ':not(#_).font': {
      fontSize: SCALES.font(1, 'inherit')
    },
    ':not(#_).mx': {
      marginLeft: SCALES.ml(0, 'revert'),
      marginRight: SCALES.mr(0, 'revert')
    },
    ':not(#_).my': {
      marginTop: SCALES.mt(0, 'revert'),
      marginBottom: SCALES.mb(0, 'revert')
    },
    ':not(#_).px': {
      paddingLeft: SCALES.pl(0, 'revert'),
      paddingRight: SCALES.pr(0, 'revert')
    },
    ':not(#_).py': {
      paddingTop: SCALES.pt(0, 'revert'),
      paddingBottom: SCALES.pb(0, 'revert')
    }
  }))

  const classes = clsx(className, classNames)

  return (
    <Component className={classes} style={style} {...props}>
      {children}
    </Component>
  )
}

export { TextChild }
