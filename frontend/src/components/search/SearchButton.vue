<script setup lang="ts">
import { Tooltip, TooltipContent, TooltipTrigger } from "../ui/tooltip";
import { Button } from "../ui/button";
import { SearchIcon } from "lucide-vue-next";
</script>

<template>
  <Tooltip>
    <TooltipTrigger as-child>
      <Button variant="outline" class="w-10 h-10">
        <SearchIcon />
      </Button>
    </TooltipTrigger>
    <TooltipContent> Search </TooltipContent>
  </Tooltip>
</template>
