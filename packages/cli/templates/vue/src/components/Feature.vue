<template>
    <div class='col background'>
        <div class="text--center">
            <img :src="props.Img" class="featureSvg" />
        </div>
        <div class="text--center">
            <h3>{{ props.title }}</h3>
            <p style="color: #333">{{ props.description }}</p>
        </div>
    </div>
</template>

<script lang="ts">
import { defineComponent } from 'vue';

export default defineComponent({
    props: {
        title: String,
        description: String,
        Img: String
    },
    setup(props) {
        return {
            props
        }
    },

})
</script>

<style></style>