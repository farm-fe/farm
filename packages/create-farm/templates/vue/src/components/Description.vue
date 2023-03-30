<template>
    <div class="description">
        <p>Supper fast web building tool written in Rust</p>
        <div style="display:flex;justifyContent:center;marginTop:40px">
            <Button to="https://farm-fe.github.io/docs/quick-start">
                Quick Start ⏱️
            </Button>
            <Button to="https://farm-fe.github.io/docs/why-farm">Why Farm?</Button>
        </div>
        <HomepageFeatures />
    </div>
</template>

<script lang="ts">
import { defineComponent } from 'vue';
import Button from './Button.vue';
import HomepageFeatures from './HomepageFeatures.vue';
export default defineComponent({
    components: {
        Button,
        HomepageFeatures,
    },
    setup() {

    }
})
</script>

<style >
.description p {
    margin: 0;
    color: #666;
}
</style>