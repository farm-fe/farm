<template>
    <div class="farm-title-wrapper">
        <img src="../../assets/logo.svg" class="farm-logo" alt="Farm Logo" />
        <h1 class="farm-title">Farm</h1>
    </div>
</template>

<script lang="ts">
import { defineComponent } from 'vue';

export default defineComponent({
})
</script>

<style>
.farm-title {
    color: rgb(113, 26, 95);
    font-size: 100px;
    font-style: italic;
    margin: 0;
}

.farm-title-wrapper {
    display: flex;
    justify-content: center;
    align-items: center;
}

.farm-logo {
    width: 150px;
}
</style>