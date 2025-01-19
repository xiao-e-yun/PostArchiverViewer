<script lang="ts" setup>
import type { InfoAPI } from '@/types';
import { ofetch } from 'ofetch';

const info = await ofetch<InfoAPI>("/api/info");
</script>

<template>
  <h1 class="text-4xl font-bold my-4">Database</h1>
  <p>Total authors: {{ info.authors }}</p>
  <p>Total posts: {{ info.posts }}</p>
  <p>Total files: {{ info.files }}</p>
</template>