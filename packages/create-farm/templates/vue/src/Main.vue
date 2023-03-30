<template>
    <Title></Title>
    <Description></Description>
</template>


<script lang="ts">
import { defineComponent } from 'vue';
import Title from "./components/Title.vue"
import Description from "./components/Description.vue"

export default defineComponent({
    setup() {

    },
    components: {
        Title,
        Description
    }
})
</script>



<style >
#app {
    font-size: 24px;
    /* color: red; */
    /* display: flex; */
    /* flex-flow: row wrap; */
    /* justify-content: center; */
    /* align-items: center; */
    height: 300px;
    text-align: center;
}
</style>

