<template>
    <a class="farm-button" :href="props.to" target="_blank">
        <slot></slot>
    </a>
</template>

<script lang="ts">
import { defineComponent } from 'vue';

export default defineComponent({
    props: {
        to: String,
    },
    setup(props) {
        return {
            props
        }
    }
})
</script>


<style>
.farm-button {
    background-color: #9f1a8f;
    color: #fff;
    padding: 10px 40px;
    border-radius: 5px;
    margin-right: 20px;
    text-decoration: none;
}

.farm-button:hover {
    background-color: #bf1aaf;
}
</style>