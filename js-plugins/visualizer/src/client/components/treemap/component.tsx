import { forwardRef,  useCallback, useEffect, useImperativeHandle, useRef } from "react";
import type {Ref} from 'react'
import { createTreemap, presetDecorator } from 'squarified'
import type { ExposedEventCallback } from 'squarified'

export type TreemapComponentInstance = ReturnType<typeof createTreemap>

export interface TreemapProps {
  onMousemove: ExposedEventCallback<'mousemove'>
}

export const Treemap = forwardRef((props: TreemapProps, ref: Ref<TreemapComponentInstance>)=>{

  const root = useRef<HTMLDivElement | null>(null)
  const instanceRef = useRef<TreemapComponentInstance>()

  const callbackRef = useCallback((el: HTMLDivElement | null)=> { 
    if (el) {
      instanceRef.current = createTreemap()
      instanceRef.current.use('decorator', presetDecorator)
      instanceRef.current.init(el)
    } else {
      instanceRef.current?.dispose()
      instanceRef.current = undefined
    }
    root.current = el
  }, [])
  

  useImperativeHandle(ref, ()=> instanceRef.current)

  useEffect(() => {
    instanceRef.current?.on('mousemove', props.onMousemove)
  }, [props])

  return <div ref={callbackRef} />
})