<script setup lang="ts">
import { Tooltip, TooltipContent, TooltipTrigger } from "../ui/tooltip";
import { categoryBuilders } from "@/category";
import { Button } from "../ui/button";
import { ref } from "vue";

const help = ref(false);
</script>

<template>
  <Tooltip v-model:open="help">
    <TooltipTrigger as-child>
      <Button
        variant="secondary"
        class="ml-auto cursor-pointer text-xs rounded-full h-6 px-2.5 capitalize"
        @click="help = true"
      >
        ?
      </Button>
    </TooltipTrigger>
    <TooltipContent align="end">
      <template v-for="builder in categoryBuilders" :key="builder.TYPE">
        <code class="bg-secondary px-1 rounded-sm text-primary">{{
          builder.PREFIX
        }}</code>
        invokes {{ builder.TYPE }}.<br />
      </template>
      Click on labels to remove them.
    </TooltipContent>
  </Tooltip>
</template>
