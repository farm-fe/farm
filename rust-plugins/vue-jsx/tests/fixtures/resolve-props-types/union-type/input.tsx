import { defineComponent } from 'vue'

interface CommonProps {
  size?: 'xl' | 'l' | 'm' | 's' | 'xs'
}

type ConditionalProps =
  | {
      color: 'normal' | 'primary' | 'secondary'
      appearance: 'normal' | 'outline' | 'text'
    }
  | {
      color: number
      appearance: 'outline'
      note: string
    }

defineComponent((props: CommonProps & ConditionalProps) => { })
