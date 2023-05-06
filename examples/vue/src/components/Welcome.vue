<template>
  <div class="farm-container">
    <img class="logo" src="../../assets/logo.png" alt="" />
    <FarmCard>
      <div class="main-desc">
        <h2 class="main-sub-title">
          Get started With
          <span class="main-content">Vue + Farm</span>
        </h2>
        <span class="main-content">
          Super fast web building tool written in Rust.
        </span>
      </div>
    </FarmCard>
    <div class="farm-desc">
      <div class="farm-desc-left">
        <FarmCard>
          <div class="container-box">
            <img src="../../assets/feature.svg" alt="" />
            <div class="rich-desc">
              <span class="sub-title">Rich Features</span>
              <span class="rich-content">
                Farm support compiling Html, Css, Js/Jsx/Ts/Tsx, Static Assets
                natively, support lazy compiling, partial bundling and
                more.</span
              >
            </div>
          </div>
        </FarmCard>
        <div class="farm-desc-right">
          <FarmCard>
            <div class="container-box">
              <img src="../../assets/plugin.svg" alt="" />
              <div class="fully-desc">
                <span class="sub-title">Fully Pluggable</span>
                <span class="fully-content">
                  Everything inside Farm is powered by plugins, you can achieve
                  anything you want by writing a plugin. Support both Rust and
                  Js plugins.</span
                >
              </div>
            </div>
          </FarmCard>
        </div>
      </div>
      <FarmCard>
        <div class="action-box">
          <div class="container-box">
            <img src="../../assets/light.svg" alt="" />
            <div class="fast-desc">
              <span class="sub-title">Super Fast </span>
              <span class="fast-content">
                Farm's compiler is written in Rust, with multi-threading,
                lazy/asynchronous compilation and persist caching, Farm can
                start a project in milliseconds, perform a HMR update within
                10ms.</span
              >
            </div>
          </div>
          <div class="action">
            <ButtonAction to="https://farm-fe.github.io/docs/quick-start"
              >Quick Start</ButtonAction
            >
            <ButtonAction to="https://farm-fe.github.io/docs/why-farm"
              >Why Farm ?</ButtonAction
            >
          </div>
        </div>
      </FarmCard>
    </div>
  </div>
</template>

<script setup lang="ts">
import FarmCard from './Card.vue';
import ButtonAction from './Button.vue';
</script>

<style scoped>
.main-content {
  font-weight: bold;
  font-size: 28px;
  background: var(--accent-gradient);
  background-clip: text;
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  color: transparent;
  background-size: 400%;
  background-position: 0%;
}
.action-box {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 40px;
  height: 100%;
}
.action {
  display: flex;
  gap: 40px;
}
.farm-container {
  display: flex;
  flex-direction: column;
  justify-content: center;
  height: 100%;
  align-items: center;
  padding: 0rem 1rem;
  gap: 1.7rem;
}
.farm-desc {
  display: flex;
  justify-content: space-between;
  gap: 32px;
}

.container-box {
  display: flex;
  align-items: center;
  gap: 2rem;
}
.container-box img {
  width: 70px;
  height: 70px;
}
.farm-desc-left {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.main-desc {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 10px;
}

.rich-desc,
.fast-desc,
.fully-desc {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.sub-title {
  font-size: 1.25rem;
  font-weight: 600;
}

.logo {
  width: 450px;
}

.img {
  width: 100px;
  height: 100px;
}

@media (max-width: 1760px) {
  .farm-desc {
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    gap: 32px;
  }
  .farm-container {
    width: 60vw;
  }
}

@media (max-width: 640px) {
  .farm-container {
    width: 100vw;
  }
}
</style>
