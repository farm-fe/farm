<template>
    <section class="features">
        <div class="container">
            <div class="row">
                <Feature v-for="(feature) in FeatureList" :Img="feature.Img" :title="feature.title"
                    :description="feature.description" :key="feature.title" />
            </div>
        </div>
    </section>
</template>
<script lang="ts">
import { defineComponent } from 'vue';
import rocket from "../../assets/rocket.svg"
import toolbox from "../../assets/toolbox.svg"
import plug from "../../assets/plug.svg"
import Feature from './Feature.vue';
export default defineComponent({
    components: {
        Feature
    },
    setup() {
        const FeatureList = [
            {
                title: 'Supper Fast',
                Img: rocket,
                description: (
                    `Farm's compiler is written in Rust, with multi-threading,
        lazy/asynchronous compilation and persist caching, Farm can start a
        project in milliseconds, perform a HMR update within 10ms.`
                )
            },
            {
                title: 'Rich Features',
                Img: toolbox,
                description: (
                    `Farm support compiling Html, Css, Js/Jsx/Ts/Tsx, Static Assets natively,
        support lazy compiling, partial bundling and more.`
                )
            },
            {
                title: 'Fully Pluggable',
                Img: plug,
                description: (
                    `Everything inside Farm is powered by plugins, you can achieve anything
        you want by writing a plugin. Support both Rust and Js plugins.`
                )
            }
        ];

        return {
            FeatureList
        }
    }
})


</script>

<style>
.features {
    display: flex;
    align-items: center;
    padding: 2rem 0;
    width: 100%;
}

.featureSvg {
    height: 100px;
    width: 100px;
}

.row {
    display: flex;
}

.col {
    flex: 1;
    padding: 1rem 2rem;
    margin: 0 1rem;
}

.col p {
    color: #333;
}

.background {
    background-color: #f5f5f5;
    border-radius: 10px;
}
</style>