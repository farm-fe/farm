<script setup>
import defer * as deferredFeature from '../deferred-feature';

const message = deferredFeature.message;
</script>

<template>
  <p id="defer-message">{{ message }}</p>
</template>
