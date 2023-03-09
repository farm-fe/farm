


<template>
  <button className='counter-button' @click="handleClick"> {{ text }} </button>
</template>

<script lang='ts'>
import { defineComponent, ref } from 'vue';

import './index.css'

const COUNT_DOWN = 60;
const BUTTON_TEXT = 'Send';


export default defineComponent({
  setup() {
    const count = ref(COUNT_DOWN)
    const text = ref(BUTTON_TEXT)
    const timer = ref(-1)
    const pause = ref(true)
    const countdown = () => {
      if (count.value === 0) {
        clearInterval(timer.value)
        text.value = BUTTON_TEXT
        count.value = COUNT_DOWN
        pause.value = true
      }
      else {
        count.value = count.value - 1
        text.value = count.value + "";

      }
    };

    const handleClick = function () {
      if (timer.value && pause.value === false) {
        pause.value = true
        text.value = 'Pause'
        clearInterval(timer.value);
      } else if (pause.value === true) {
        pause.value = false
        text.value = `${count.value}`
        timer.value = setInterval(countdown, 1000)
      }
      else {
        const t = setInterval(countdown, 1000);
        text.value = COUNT_DOWN + ""
        timer.value = t
      }
    }

    return {
      handleClick,
      count,
      text,
      timer,
      pause,
      countdown
    }
  }

})

</script>