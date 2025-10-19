<script setup lang="ts">
import { provide } from "vue";
import { Card } from "../ui/card";
import { TooltipProvider } from "../ui/tooltip";
import { commitRef } from "@/utils";
import SearchHelp from "./SearchHelp.vue";
import { SearchContextKey, type SearchQuerys } from "./search";
import SearchSelected from "./SearchSelected.vue";
import SearchButton from "./SearchButton.vue";
import SearchCombobox from "./SearchCombobox.vue";

const model = defineModel<SearchQuerys>({ required: true });
const querys = commitRef(model);

provide(SearchContextKey, querys);

const commit = querys.commit;
</script>

<template>
  <Card class="p-2">
    <TooltipProvider>
      <form class="flex gap-2" @submit.prevent="commit">
        <SearchCombobox />
        <SearchButton />
      </form>
      <div class="flex gap-2 mt-2 flex-wrap">
        <SearchSelected />
        <SearchHelp />
      </div>
    </TooltipProvider>
  </Card>
</template>
