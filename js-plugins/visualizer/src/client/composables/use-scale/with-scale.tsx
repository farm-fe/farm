import React, { forwardRef } from 'react'
import { ScaleProvider } from './scale-context'
import type { DynamicLayoutPipe, ScaleProps } from './scale-context'
import { generateGetAllScaleProps, generateGetScaleProps } from './utils'

function isCSSNumberValue(value?: string | number) {
  return value !== undefined && !Number.isNaN(+value)
}

function reduceScaleCoefficient(scale: number) {
  if (scale === 1) { return scale }
  const diff = Math.abs((scale - 1) / 2)
  return scale > 1 ? 1 + diff : 1 - diff
}

type AttrValue = string | number | undefined

function makeScaleHandler(attrValue: AttrValue, unit: string, scale: number): DynamicLayoutPipe {
  return (scale1x, defaultValue) => {
    // 0 means disable scale and the default value is 0
    if (scale1x === 0) {
      scale1x = 1
      defaultValue = defaultValue || 0
    }
    const factor = reduceScaleCoefficient(scale) * scale1x
    if (typeof attrValue === 'undefined') {
      if (typeof defaultValue !== 'undefined') { return `${defaultValue}` }
      return `calc(${factor} * ${unit})`
    }

    if (!isCSSNumberValue(attrValue)) { return `${attrValue}` }
    const customFactor = factor * Number(attrValue)
    return `calc(${customFactor} * ${unit})`
  }
}

export const withScale = <T, P = Empty>(
  Render: React.ComponentType<P & { ref?: React.Ref<T> }> | React.ForwardRefExoticComponent<P>
) => {
  const ScaleFC = forwardRef<T, P & React.PropsWithChildren<ScaleProps>>(({ children, ...props }, ref) => {
    const {
      paddingLeft,
      pl,
      paddingRight,
      pr,
      paddingTop,
      pt,
      paddingBottom,
      pb,
      marginTop,
      mt,
      marginRight,
      mr,
      marginBottom,
      mb,
      marginLeft,
      ml,
      px,
      py,
      mx,
      my,
      width,
      height,
      font,
      w,
      h,
      margin,
      padding,
      unit = '16px',
      scale = 1,
      ...innerProps
    } = props

    const scaleConfig = {
      unit: '29px',
      SCALES: {
        pt: makeScaleHandler(paddingTop ?? pt ?? py ?? padding, unit, scale),
        pr: makeScaleHandler(paddingRight ?? pr ?? px ?? padding, unit, scale),
        pb: makeScaleHandler(paddingBottom ?? pb ?? py ?? padding, unit, scale),
        pl: makeScaleHandler(paddingLeft ?? pl ?? px ?? padding, unit, scale),
        px: makeScaleHandler(px ?? paddingLeft ?? paddingRight ?? pl ?? pr ?? padding, unit, scale),
        py: makeScaleHandler(py ?? paddingTop ?? paddingBottom ?? pt ?? pb ?? padding, unit, scale),
        mt: makeScaleHandler(marginTop ?? mt ?? my ?? margin, unit, scale),
        mr: makeScaleHandler(marginRight ?? mr ?? mx ?? margin, unit, scale),
        mb: makeScaleHandler(marginBottom ?? mb ?? my ?? margin, unit, scale),
        ml: makeScaleHandler(marginLeft ?? ml ?? mx ?? margin, unit, scale),
        mx: makeScaleHandler(mx ?? marginLeft ?? marginRight ?? ml ?? mr ?? margin, unit, scale),
        my: makeScaleHandler(my ?? marginTop ?? marginBottom ?? mt ?? mb ?? margin, unit, scale),
        width: makeScaleHandler(width ?? w, unit, scale),
        height: makeScaleHandler(height ?? h, unit, scale),
        font: makeScaleHandler(font, unit, scale)
      },
      getScaleProps: generateGetScaleProps(props),
      getAllScaleProps: generateGetAllScaleProps(props)
    }

    return (
      <ScaleProvider value={scaleConfig}>
        <Render {...(innerProps as P)} ref={ref}>
          {children}
        </Render>
      </ScaleProvider>
    )
  })
  return ScaleFC
}
