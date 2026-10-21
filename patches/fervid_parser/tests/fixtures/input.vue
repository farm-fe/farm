<template>
  <abc-def
    v-model.lazy="modelValue"
    v-model:another-model-value.trim="modelValue"
    v-test-directive:test-argument.foo.bar="foo - bar"
    :test-bound="bar + baz"
    disabled
    class=""
    @click.prevent
    @hello="world"
  >
    The text of the node

    {{ dynamicValue }}

    <slot></slot>

    <slot name="named-slot-1"></slot>

    <slot name="named-slot-2" class="ye" :prop="modelValue">
      <div>
        default content
      </div>
    </slot>

    <!-- Comment -->
    <another-element></another-element>

    yet another text

    <template v-slot:test-slot="{ value, another: renamed }">
      test {{ value }} {{ renamed }}
    </template>

    <template #custom-slot="prop">
      <span class="span-class">
        Span text
      </span>
      {{ prop.nested }}
    </template>

    <input v-model="inputModel" v-directive:foo.bar.buzz="baz">

    <!-- Todo remove space between these elements, otherwise it breaks the invariant in conditional codegen -->
    <div v-if="true">if div</div>
    <span v-else-if="false">else-if span</span>

    <span v-for="i in list" :key="i">hey</span>
    <br v-show="false">
    <another-element v-for="i in 3" :key="i" v-text="foo + bar"></another-element>

    <template v-for="([item1, item2], index) in list" :key="index">
      hey
      <span>{{ item1 }}{{ index }}</span>
      <div class="both regular and bound" :class="[item2, index]"></div>
      <div class="just regular class"></div>
      <div :class="[member.expr, globalIdent, item2, index]"></div>
    </template>

    <template v-if="false">
      this is a v-if template
    </template>

    <template v-else-if="true">
    	another v-else-if template
    </template>

    <template v-else>
    	else template
    </template>

    <div
      v-text="foo + bar"
      style="background-color:red;color:#000;content: ''; grid-template-column: repeat(1fr, min(auto-fit, 100px))"
      :style="{ backgroundColor: v ? 'yellow' : undefined }"
    ></div>
  </abc-def>
</template>

<script setup>
import { ref } from 'vue'

const foo = '123'
const bar = ref(456)
const inputModel = ref('')
const list = [1, 2, 3]

defineProps({ baz: String })
defineEmits(['emit-change'])
defineExpose('foo', 'bar')

const modelValue = defineModel()
</script>

<style scoped>
.span-class {
  background: yellow;
}
</style>
