<script setup lang="ts">
import defer * as deferredFeature from '../deferred-feature';

const message = deferredFeature.message;
</script>

<template>
  <p id="defer-message">{{ message }}</p>
</template>
