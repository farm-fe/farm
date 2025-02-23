import React from 'react'
import { useScale, withScale } from '../../composables'

interface Props {
  inline?: boolean
}

export type SpacerProps = Omit<React.HTMLAttributes<unknown>, keyof Props> & Props

function SpacerComponent({ inline = false, ...props }: SpacerProps) {
  const { SCALES } = useScale()
  return (
    <div
      stylex={{
        width: SCALES.width(1),
        height: SCALES.height(1),
        padding: `${SCALES.pt(0)} ${SCALES.pr(0)} ${SCALES.pb(0)} ${SCALES.pl(0)}`,
        margin: `${SCALES.mt(0)} ${SCALES.mr(0)} ${SCALES.mb(0)} ${SCALES.ml(0)}`,
        display: 'block',
        ...(inline && { display: 'inline-block' })
      }}
      {...props}
    />
  )
}

export const Spacer = withScale(SpacerComponent)
