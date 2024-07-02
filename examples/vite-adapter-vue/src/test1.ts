import { Type, type Static } from '@sinclair/typebox'
import { Value } from '@sinclair/typebox/value'

const T = Type.Object({                              // const T = {
  id: Type.String(),                                 //   type: 'object',
  name: Type.String(),                               //   properties: {
  timestamp: Type.Integer()                          //     id: {
}) 

type T = Static<typeof T>  

export function receive(value: T) {                         // ... as a Static Type

  if(Value.Check(T, value)) {                        // ... as a Json Schema
    console.log('ok')
    // ok...
  }
}